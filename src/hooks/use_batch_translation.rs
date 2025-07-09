use crate::error::{use_error_handler, AppError};
use crate::services::{
    batch_service::{
        BatchProgress, BatchStatus, BatchTranslationService, DocumentLink, TranslatedDocument,
    },
    config_service::ConfigService,
    history_service::HistoryService,
};
use crate::types::history::{BatchDocumentInfo, BatchTranslationData, HistoryEntry};
use leptos::*;
use wasm_bindgen_futures::spawn_local;

pub struct UseBatchTranslationReturn {
    pub is_processing: ReadSignal<bool>,
    pub progress: ReadSignal<BatchProgress>,
    pub documents: ReadSignal<Vec<DocumentLink>>,
    pub translated_docs: ReadSignal<Vec<TranslatedDocument>>,
    pub start_batch_translation: WriteSignal<Option<String>>,
}

pub fn use_batch_translation() -> UseBatchTranslationReturn {
    let error_handler = use_error_handler();

    let (is_processing, set_is_processing) = create_signal(false);
    let (progress, set_progress) = create_signal(BatchProgress {
        total: 0,
        completed: 0,
        current_task: String::new(),
        failed_count: 0,
        status: BatchStatus::Idle,
    });
    let (documents, set_documents) = create_signal(Vec::<DocumentLink>::new());
    let (translated_docs, set_translated_docs) = create_signal(Vec::<TranslatedDocument>::new());
    let (start_trigger, set_start_trigger) = create_signal(None::<String>);

    // 批量翻译处理Effect
    create_effect({
        let set_is_processing = set_is_processing.clone();
        let set_progress = set_progress.clone();
        let set_documents = set_documents.clone();
        let set_translated_docs = set_translated_docs.clone();

        move |_| {
            if let Some(index_url) = start_trigger.get() {
                if index_url.is_empty() {
                    error_handler
                        .handle_error(AppError::validation("URL", "请输入有效的文档索引URL"));
                    return;
                }

                set_is_processing.set(true);
                set_translated_docs.set(Vec::new());
                set_progress.set(BatchProgress {
                    total: 0,
                    completed: 0,
                    current_task: "开始处理...".to_string(),
                    failed_count: 0,
                    status: BatchStatus::Parsing,
                });

                let set_progress_clone = set_progress.clone();
                let set_is_processing_clone = set_is_processing.clone();
                let set_documents_clone = set_documents.clone();
                let set_translated_docs_clone = set_translated_docs.clone();

                spawn_local(async move {
                    web_sys::console::log_1(&"=== 开始批量翻译流程 ===".into());
                    web_sys::console::log_1(&format!("索引URL: {}", index_url).into());

                    let config_service = ConfigService::new();

                    match config_service.get_config() {
                        Ok(config) => {
                            web_sys::console::log_1(&"配置加载成功".into());
                            let batch_service = BatchTranslationService::new(&config);

                            // 步骤1: 解析文档索引
                            web_sys::console::log_1(&"=== 步骤1: 解析文档索引 ===".into());
                            set_progress_clone.set(BatchProgress {
                                total: 0,
                                completed: 0,
                                current_task: "正在解析文档索引...".to_string(),
                                failed_count: 0,
                                status: BatchStatus::Parsing,
                            });

                            match batch_service.parse_document_index(&index_url).await {
                                Ok(links) => {
                                    web_sys::console::log_1(
                                        &format!("索引解析成功，找到 {} 个文档", links.len())
                                            .into(),
                                    );
                                    set_documents_clone.set(links.clone());

                                    // 步骤2: 批量翻译
                                    web_sys::console::log_1(&"=== 步骤2: 开始批量翻译 ===".into());

                                    let total_links = links.len(); // 保存总数

                                    let progress_callback = {
                                        let set_progress_clone = set_progress_clone.clone();
                                        move |progress: BatchProgress| {
                                            set_progress_clone.set(progress);
                                        }
                                    };

                                    match batch_service
                                        .batch_translate(links, progress_callback)
                                        .await
                                    {
                                        Ok(translated_documents) => {
                                            web_sys::console::log_1(
                                                &format!(
                                                    "批量翻译完成，成功翻译 {} 个文档",
                                                    translated_documents.len()
                                                )
                                                .into(),
                                            );
                                            set_translated_docs_clone
                                                .set(translated_documents.clone());

                                            // 步骤3: 创建ZIP文件
                                            web_sys::console::log_1(
                                                &"=== 步骤3: 创建ZIP文件 ===".into(),
                                            );
                                            set_progress_clone.set(BatchProgress {
                                                total: translated_documents.len(),
                                                completed: translated_documents.len(),
                                                current_task: "正在打包文件...".to_string(),
                                                failed_count: 0,
                                                status: BatchStatus::Packaging,
                                            });

                                            match batch_service
                                                .create_compressed_archive(&translated_documents)
                                            {
                                                Ok(compressed_data) => {
                                                    web_sys::console::log_1(
                                                        &"tar.gz文件创建成功".into(),
                                                    );

                                                    // 保存到历史记录
                                                    web_sys::console::log_1(
                                                        &"=== 步骤4: 保存历史记录 ===".into(),
                                                    );
                                                    let history_service = HistoryService::new();

                                                    // 创建批量翻译数据
                                                    let batch_document_list: Vec<
                                                        BatchDocumentInfo,
                                                    > = translated_documents
                                                        .iter()
                                                        .map(|doc| BatchDocumentInfo {
                                                            title: doc.link.title.clone(),
                                                            url: doc.link.url.clone(),
                                                            file_name: doc.file_name.clone(),
                                                            folder_path: doc.folder_path.clone(),
                                                            order: doc.link.order,
                                                            translated: true,
                                                            original_content: doc
                                                                .original_content
                                                                .clone(),
                                                            translated_content: doc
                                                                .translated_content
                                                                .clone(),
                                                        })
                                                        .collect();

                                                    let failed_count =
                                                        total_links - translated_documents.len();
                                                    let batch_data = BatchTranslationData {
                                                        total_documents: total_links,
                                                        successful_documents: translated_documents
                                                            .len(),
                                                        failed_documents: failed_count,
                                                        index_url: index_url.clone(),
                                                        document_list: batch_document_list,
                                                    };

                                                    let title = format!(
                                                        "批量翻译: {}",
                                                        if let Some(first_doc) =
                                                            translated_documents.first()
                                                        {
                                                            extract_domain_from_url(
                                                                &first_doc.link.url,
                                                            )
                                                        } else {
                                                            "未知网站".to_string()
                                                        }
                                                    );

                                                    let history_entry =
                                                        HistoryEntry::new_batch_translation(
                                                            index_url.clone(),
                                                            title,
                                                            config.default_source_lang.clone(),
                                                            config.default_target_lang.clone(),
                                                            batch_data,
                                                        );

                                                    if let Err(e) =
                                                        history_service.add_entry(history_entry)
                                                    {
                                                        web_sys::console::log_1(
                                                            &format!("保存历史记录失败: {}", e)
                                                                .into(),
                                                        );
                                                    } else {
                                                        web_sys::console::log_1(
                                                            &"批量翻译历史记录保存成功".into(),
                                                        );
                                                    }

                                                    // 生成智能压缩文件名
                                                    let archive_name =
                                                        generate_archive_name(&index_url);

                                                    // 触发下载
                                                    if let Err(e) = trigger_download(
                                                        &compressed_data,
                                                        &archive_name,
                                                    ) {
                                                        web_sys::console::log_1(
                                                            &format!("下载失败: {}", e).into(),
                                                        );
                                                        error_handler.handle_error(AppError::file(
                                                            format!("下载失败: {}", e),
                                                        ));
                                                    }

                                                    set_progress_clone.set(BatchProgress {
                                                        total: translated_documents.len(),
                                                        completed: translated_documents.len(),
                                                        current_task: "批量翻译完成".to_string(),
                                                        failed_count: 0,
                                                        status: BatchStatus::Completed,
                                                    });

                                                    web_sys::console::log_1(
                                                        &"=== 批量翻译流程完成 ===".into(),
                                                    );
                                                }
                                                Err(e) => {
                                                    web_sys::console::log_1(
                                                        &format!("ZIP创建失败: {}", e).into(),
                                                    );
                                                    let error_msg = format!("打包失败: {}", e);
                                                    set_progress_clone.set(BatchProgress {
                                                        total: 0,
                                                        completed: 0,
                                                        current_task: error_msg.clone(),
                                                        failed_count: 0,
                                                        status: BatchStatus::Failed(
                                                            error_msg.clone(),
                                                        ),
                                                    });
                                                    error_handler
                                                        .handle_error(AppError::file(error_msg));
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            web_sys::console::log_1(
                                                &format!("批量翻译失败: {}", e).into(),
                                            );
                                            let error_msg = format!("批量翻译失败: {}", e);
                                            set_progress_clone.set(BatchProgress {
                                                total: 0,
                                                completed: 0,
                                                current_task: error_msg.clone(),
                                                failed_count: 0,
                                                status: BatchStatus::Failed(error_msg.clone()),
                                            });
                                            error_handler
                                                .handle_error(AppError::translation(error_msg));
                                        }
                                    }
                                }
                                Err(e) => {
                                    web_sys::console::log_1(&format!("索引解析失败: {}", e).into());
                                    let error_msg = format!("索引解析失败: {}", e);
                                    set_progress_clone.set(BatchProgress {
                                        total: 0,
                                        completed: 0,
                                        current_task: error_msg.clone(),
                                        failed_count: 0,
                                        status: BatchStatus::Failed(error_msg.clone()),
                                    });
                                    error_handler.handle_error(AppError::extraction(error_msg));
                                }
                            }
                        }
                        Err(e) => {
                            web_sys::console::log_1(&format!("配置加载失败: {}", e).into());
                            let error_msg = format!("配置加载失败: {}", e);
                            set_progress_clone.set(BatchProgress {
                                total: 0,
                                completed: 0,
                                current_task: error_msg.clone(),
                                failed_count: 0,
                                status: BatchStatus::Failed(error_msg.clone()),
                            });
                            error_handler.handle_error(AppError::config(error_msg));
                        }
                    }

                    set_is_processing_clone.set(false);
                });
            }
        }
    });

    UseBatchTranslationReturn {
        is_processing,
        progress,
        documents,
        translated_docs,
        start_batch_translation: set_start_trigger,
    }
}

/// 触发文件下载
fn trigger_download(data: &[u8], filename: &str) -> Result<(), String> {
    use wasm_bindgen::JsCast;

    let window = web_sys::window().ok_or("无法获取window对象")?;
    let document = window.document().ok_or("无法获取document对象")?;

    // 创建Blob
    let array = js_sys::Uint8Array::new_with_length(data.len() as u32);
    array.copy_from(data);

    let blob_parts = js_sys::Array::new();
    blob_parts.push(&array);

    let blob =
        web_sys::Blob::new_with_u8_array_sequence(&blob_parts).map_err(|_| "无法创建Blob对象")?;

    // 创建下载链接
    let url = web_sys::Url::create_object_url_with_blob(&blob).map_err(|_| "无法创建对象URL")?;

    let anchor = document
        .create_element("a")
        .map_err(|_| "无法创建a元素")?
        .dyn_into::<web_sys::HtmlAnchorElement>()
        .map_err(|_| "无法转换为HtmlAnchorElement")?;

    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor
        .style()
        .set_property("display", "none")
        .map_err(|_| "无法设置样式")?;

    let body = document.body().ok_or("无法获取body元素")?;
    body.append_child(&anchor).map_err(|_| "无法添加下载链接")?;

    // 触发点击下载
    anchor.click();

    // 清理
    body.remove_child(&anchor).map_err(|_| "无法移除下载链接")?;
    web_sys::Url::revoke_object_url(&url).map_err(|_| "无法释放对象URL")?;

    Ok(())
}

/// 生成智能压缩文件名：域名_时间戳.tar.gz
fn generate_archive_name(url: &str) -> String {
    let domain = extract_domain_from_url(url);
    let clean_domain = clean_domain_name(&domain);
    let timestamp = js_sys::Date::new_0()
        .to_iso_string()
        .as_string()
        .unwrap_or_else(|| "unknown".to_string());
    let clean_timestamp = timestamp
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect::<String>()
        .replace('T', "_")
        .split('.')
        .next()
        .unwrap_or("unknown")
        .to_string();

    format!("{}_{}.tar.gz", clean_domain, clean_timestamp)
}

/// 清理域名，移除特殊字符
fn clean_domain_name(domain: &str) -> String {
    domain
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

/// 从URL提取域名
fn extract_domain_from_url(url: &str) -> String {
    if let Ok(parsed_url) = web_sys::Url::new(url) {
        let hostname = parsed_url.hostname();
        if !hostname.is_empty() {
            return hostname;
        }
    }

    // 备用方案：手动解析
    if let Some(start) = url.find("://") {
        let after_protocol = &url[start + 3..];
        if let Some(end) = after_protocol.find('/') {
            after_protocol[..end].to_string()
        } else {
            after_protocol.to_string()
        }
    } else {
        "unknown".to_string()
    }
}
