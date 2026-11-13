//! `Client` do CEF: recebe eventos de cada aba (título, endereço,
//! carregamento, erros, popups, downloads, certificados) e repassa à UI.

use cef::*;
use osmarcio_core::web;

fn s(v: Option<&CefString>) -> String {
    v.map(CefString::to_string).unwrap_or_default()
}

fn id(b: &Option<&mut Browser>) -> Option<i32> {
    b.as_ref().map(|b| b.identifier())
}

wrap_client! {
    pub struct OsmClient;

    impl Client {
        fn display_handler(&self) -> Option<DisplayHandler> { Some(OsmDisplay::new()) }
        fn load_handler(&self) -> Option<LoadHandler> { Some(OsmLoad::new()) }
        fn life_span_handler(&self) -> Option<LifeSpanHandler> { Some(OsmLifeSpan::new()) }
        fn download_handler(&self) -> Option<DownloadHandler> { Some(OsmDownload::new()) }
        fn request_handler(&self) -> Option<RequestHandler> { Some(OsmRequest::new()) }
    }
}

wrap_display_handler! {
    struct OsmDisplay;

    impl DisplayHandler {
        fn on_title_change(&self, browser: Option<&mut Browser>, title: Option<&CefString>) {
            if let Some(b) = id(&browser) {
                crate::ui::on_title(b, s(title));
            }
        }

        fn on_address_change(&self, browser: Option<&mut Browser>, frame: Option<&mut Frame>, url: Option<&CefString>) {
            let main = frame.map(|f| f.is_main() == 1).unwrap_or(false);
            if let (true, Some(b)) = (main, id(&browser)) {
                crate::ui::on_address(b, s(url));
            }
        }
    }
}

wrap_load_handler! {
    struct OsmLoad;

    impl LoadHandler {
        fn on_loading_state_change(&self, browser: Option<&mut Browser>, is_loading: i32, can_go_back: i32, can_go_forward: i32) {
            if let Some(b) = id(&browser) {
                crate::ui::on_loading(b, is_loading == 1, can_go_back == 1, can_go_forward == 1);
            }
        }

        fn on_load_end(&self, browser: Option<&mut Browser>, frame: Option<&mut Frame>, http_status_code: i32) {
            let Some(frame) = frame else { return };
            if frame.is_main() != 1 {
                return;
            }
            let url = CefString::from(&frame.url()).to_string();
            log::info!("osmb: load_end status={http_status_code} url={url}");
            if let Some(b) = id(&browser) {
                crate::ui::on_load_end(b, &url, http_status_code);
            }
            // Autoteste: prova que o JavaScript e o DOM funcionam, sem servidor próprio.
            if crate::ui::selftest() && url.starts_with("http") {
                let js = "localStorage.setItem('osmb','ok'); document.cookie='osmb=1; SameSite=Lax; Secure'; \
                          document.title = 'OSMB-JS:' + (6*7) + ':dom=' + document.querySelectorAll('*').length \
                          + ':ls=' + localStorage.getItem('osmb') + ':ua=' + (navigator.userAgent.includes('Chrome') ? 'chromium' : 'x');";
                frame.execute_java_script(Some(&CefString::from(js)), Some(&CefString::from("osmarcio://selftest")), 0);
            }
        }

        fn on_load_error(&self, _browser: Option<&mut Browser>, frame: Option<&mut Frame>, error_code: Errorcode, error_text: Option<&CefString>, failed_url: Option<&CefString>) {
            let code = sys::cef_errorcode_t::from(error_code) as i32;
            let Some(frame) = frame else { return };
            // -3 = ERR_ABORTED (ex.: navegação substituída ou download): não é erro.
            if frame.is_main() != 1 || code == -3 {
                return;
            }
            let (url, text) = (s(failed_url), s(error_text));
            log::warn!("osmb: load_error code={code} text={text} url={url}");
            let page = web::data_url(&web::error_page(&url, code, &text));
            frame.load_url(Some(&CefString::from(page.as_str())));
        }
    }
}

wrap_life_span_handler! {
    struct OsmLifeSpan;

    impl LifeSpanHandler {
        fn on_before_popup(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            _popup_id: i32,
            target_url: Option<&CefString>,
            _target_frame_name: Option<&CefString>,
            _target_disposition: WindowOpenDisposition,
            _user_gesture: i32,
            _popup_features: Option<&PopupFeatures>,
            _window_info: Option<&mut WindowInfo>,
            _client: Option<&mut Option<Client>>,
            _settings: Option<&mut BrowserSettings>,
            _extra_info: Option<&mut Option<DictionaryValue>>,
            _no_javascript_access: Option<&mut i32>,
        ) -> i32 {
            // Popups (target=_blank, window.open) viram abas do OSMarcio.
            let url = s(target_url);
            log::info!("osmb: popup -> nova aba {url}");
            crate::ui::post_new_tab(url);
            1 // cancela a janela popup do Chromium
        }

        fn on_after_created(&self, browser: Option<&mut Browser>) {
            if let Some(b) = id(&browser) {
                log::info!("osmb: browser criado id={b}");
            }
        }

        fn on_before_close(&self, browser: Option<&mut Browser>) {
            if let Some(b) = id(&browser) {
                crate::ui::on_browser_closed(b);
            }
        }
    }
}

wrap_download_handler! {
    struct OsmDownload;

    impl DownloadHandler {
        fn can_download(&self, _browser: Option<&mut Browser>, url: Option<&CefString>, _method: Option<&CefString>) -> i32 {
            // Somente HTTP(S): nada de baixar de esquemas internos/arquivos locais.
            let u = s(url).to_ascii_lowercase();
            (u.starts_with("https://") || u.starts_with("http://")).into()
        }

        fn on_before_download(&self, _browser: Option<&mut Browser>, _item: Option<&mut DownloadItem>, suggested_name: Option<&CefString>, callback: Option<&mut BeforeDownloadCallback>) -> i32 {
            let Some(cb) = callback else { return 0 };
            let path = crate::ui::download_path(&s(suggested_name));
            log::info!("osmb: download_start path={}", path.display());
            cb.cont(Some(&CefString::from(path.to_string_lossy().as_ref())), 0);
            1
        }

        fn on_download_updated(&self, _browser: Option<&mut Browser>, item: Option<&mut DownloadItem>, _callback: Option<&mut DownloadItemCallback>) {
            let Some(item) = item else { return };
            if item.is_complete() == 1 {
                let p = CefString::from(&item.full_path()).to_string();
                log::info!("osmb: download_complete path={p}");
                crate::ui::notify_status(&format!("Download concluído: {p}"));
            }
        }
    }
}

wrap_request_handler! {
    struct OsmRequest;

    impl RequestHandler {
        fn on_certificate_error(&self, _browser: Option<&mut Browser>, cert_error: Errorcode, request_url: Option<&CefString>, _ssl_info: Option<&mut Sslinfo>, _callback: Option<&mut Callback>) -> i32 {
            // Certificados inválidos NUNCA são aceitos: 0 = cancelar a requisição.
            let code = sys::cef_errorcode_t::from(cert_error) as i32;
            log::warn!("osmb: certificado rejeitado code={code} url={}", s(request_url));
            0
        }
    }
}
