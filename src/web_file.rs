//! Web端文件处理适配

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{File, FileList, HtmlInputElement, FileReader, Blob, Url, HtmlAnchorElement};
#[cfg(target_arch = "wasm32")]
use js_sys::{Array, Uint8Array, Function, Promise};

/// Web端文件读取器
#[cfg(target_arch = "wasm32")]
pub struct WebFileReader;

#[cfg(target_arch = "wasm32")]
impl WebFileReader {
    /// 从HTML文件输入元素读取文件
    pub async fn read_file_from_input(input: &HtmlInputElement) -> Result<Vec<u8>, String> {
        let files = input.files().ok_or("无法获取文件列表".to_string())?;
        
        if files.length() == 0 {
            return Err("没有选择文件".to_string());
        }
        
        let file = files.get(0).ok_or("无法获取文件".to_string())?;
        Self::read_file(&file).await
    }
    
    /// 读取单个文件
    pub async fn read_file(file: &File) -> Result<Vec<u8>, String> {
        let promise = Promise::new(&mut |resolve: Function, reject: Function| {
            let reader = match FileReader::new() {
                Ok(r) => r,
                Err(e) => {
                    log::error!("无法创建FileReader: {:?}", e);
                    let _ = reject.call1(&JsValue::NULL, &JsValue::from_str("无法创建FileReader"));
                    return;
                }
            };
            
            // 克隆reject以便在多个闭包中使用
            let reject_clone = reject.clone();
            let reject_clone2 = reject.clone();
            
            let reader_clone = reader.clone();
            let onload = Closure::wrap(Box::new(move || {
                match reader_clone.result() {
                    Ok(result) => {
                        let array = Uint8Array::new(&result);
                        let vec = array.to_vec();
                        let _ = resolve.call1(&JsValue::NULL, &JsValue::from(vec));
                    }
                    Err(e) => {
                        log::error!("读取文件结果错误: {:?}", e);
                        let _ = reject_clone.call1(&JsValue::NULL, &JsValue::from_str("读取文件失败"));
                    }
                }
            }) as Box<dyn Fn()>);
            
            let onerror = Closure::wrap(Box::new(move || {
                log::error!("FileReader发生错误");
                let _ = reject_clone2.call1(&JsValue::NULL, &JsValue::from_str("读取文件失败"));
            }) as Box<dyn Fn()>);
            
            reader.set_onload(Some(onload.as_ref().unchecked_ref()));
            reader.set_onerror(Some(onerror.as_ref().unchecked_ref()));
            
            onload.forget();
            onerror.forget();
            
            if let Err(e) = reader.read_as_array_buffer(file) {
                log::error!("读取文件失败: {:?}", e);
                let _ = reject.call1(&JsValue::NULL, &JsValue::from_str("读取文件失败"));
            }
        });
        
        match wasm_bindgen_futures::JsFuture::from(promise).await {
            Ok(js_value) => {
                let array = Uint8Array::new(&js_value);
                Ok(array.to_vec())
            }
            Err(e) => {
                log::error!("JsFuture错误: {:?}", e);
                Err("读取文件失败".to_string())
            }
        }
    }
    
    /// 读取多个文件
    pub async fn read_multiple_files(files: &FileList) -> Result<Vec<(String, Vec<u8>)>, String> {
        let mut results = Vec::new();
        
        for i in 0..files.length() {
            let file = files.get(i).ok_or(format!("无法获取文件 {}", i))?;
            let name = file.name();
            let data = Self::read_file(&file).await?;
            results.push((name, data));
        }
        
        Ok(results)
    }
    
    /// 创建文件下载
    pub fn download_file(data: &[u8], filename: &str, mime_type: &str) -> Result<(), String> {
        let array = Uint8Array::from(data);
        let options = web_sys::BlobPropertyBag::new();
        options.set_type(mime_type);
        
        let blob = Blob::new_with_u8_array_sequence_and_options(
            &Array::of1(&array),
            &options,
        ).map_err(|e| {
            log::error!("创建Blob失败: {:?}", e);
            "创建Blob失败".to_string()
        })?;
        
        let url = Url::create_object_url_with_blob(&blob)
            .map_err(|e| {
                log::error!("创建URL失败: {:?}", e);
                "创建URL失败".to_string()
            })?;
        
        let window = web_sys::window().ok_or("无法获取window".to_string())?;
        let document = window.document().ok_or("无法获取document".to_string())?;
        
        let a = document.create_element("a")
            .map_err(|e| {
                log::error!("创建a元素失败: {:?}", e);
                "创建a元素失败".to_string()
            })?;
        a.set_attribute("href", &url)
            .map_err(|e| {
                log::error!("设置href失败: {:?}", e);
                "设置href失败".to_string()
            })?;
        a.set_attribute("download", filename)
            .map_err(|e| {
                log::error!("设置download失败: {:?}", e);
                "设置download失败".to_string()
            })?;
        a.set_attribute("style", "display: none")
            .map_err(|e| {
                log::error!("设置style失败: {:?}", e);
                "设置style失败".to_string()
            })?;
        
        let body = document.body().ok_or("无法获取body".to_string())?;
        body.append_child(&a)
            .map_err(|e| {
                log::error!("添加元素失败: {:?}", e);
                "添加元素失败".to_string()
            })?;
        
        // 克隆元素以便在移动后仍然可以引用
        let a_clone = a.clone();
        let anchor_result = a.dyn_into::<HtmlAnchorElement>()
            .map_err(|e| {
                log::error!("转换元素失败: {:?}", e);
                "转换元素失败".to_string()
            });
        
        if let Ok(anchor) = anchor_result {
            anchor.click();
        }
        
        // 清理
        let _ = body.remove_child(&a_clone);
        let _ = Url::revoke_object_url(&url);
        
        Ok(())
    }
}

/// 桌面端文件读取器（占位符）
#[cfg(not(target_arch = "wasm32"))]
pub struct WebFileReader;

#[cfg(not(target_arch = "wasm32"))]
impl WebFileReader {
    /// 桌面端不支持此功能
    pub async fn read_file_from_input(_input: &()) -> Result<Vec<u8>, String> {
        Err("此功能仅在Web端可用".to_string())
    }
    
    /// 桌面端不支持此功能
    pub async fn read_file(_file: &()) -> Result<Vec<u8>, String> {
        Err("此功能仅在Web端可用".to_string())
    }
    
    /// 桌面端不支持此功能
    pub async fn read_multiple_files(_files: &()) -> Result<Vec<(String, Vec<u8>)>, String> {
        Err("此功能仅在Web端可用".to_string())
    }
    
    /// 桌面端不支持此功能
    pub fn download_file(_data: &[u8], _filename: &str, _mime_type: &str) -> Result<(), String> {
        Err("此功能仅在Web端可用".to_string())
    }
}
