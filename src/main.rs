use std::sync::{Arc, Mutex};
use webview2_com::Microsoft::Web::WebView2::Win32::*;
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::Com::*,
        System::LibraryLoader::GetModuleHandleW,
        UI::Controls::*,
        UI::WindowsAndMessaging::*,
    },
};

const WINDOW_WIDTH: i32 = 1200;
const WINDOW_HEIGHT: i32 = 800;
const URL_BAR_HEIGHT: i32 = 40;
const BUTTON_WIDTH: i32 = 80;
const BUTTON_HEIGHT: i32 = 30;
const MARGIN: i32 = 5;

const ID_BACK: u16 = 1001;
const ID_FORWARD: u16 = 1002;
const ID_REFRESH: u16 = 1003;
const ID_GO: u16 = 1004;
const ID_URL_BAR: u16 = 1005;

struct BrowserState {
    webview_controller: Option<ICoreWebView2Controller>,
    webview: Option<ICoreWebView2>,
    url_bar: HWND,
}

impl BrowserState {
    fn new() -> Self {
        Self {
            webview_controller: None,
            webview: None,
            url_bar: HWND(0),
        }
    }
}

fn main() -> Result<()> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED)?;

        let instance = GetModuleHandleW(None)?;
        let window_class = w!("RustBrowserWindow");

        let wc = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            lpszClassName: window_class,
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hbrBackground: HBRUSH((COLOR_WINDOW.0 + 1) as isize),
            ..Default::default()
        };

        RegisterClassW(&wc);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            window_class,
            w!("Rust Browser"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            None,
            None,
            instance,
            None,
        )?;

        ShowWindow(hwnd, SW_SHOW);

        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).into() {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }

        CoUninitialize();
        Ok(())
    }
}

unsafe extern "system" fn wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            // Create navigation buttons
            let back_btn = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("BUTTON"),
                w!("← Back"),
                WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_PUSHBUTTON as u32),
                MARGIN,
                MARGIN,
                BUTTON_WIDTH,
                BUTTON_HEIGHT,
                hwnd,
                HMENU(ID_BACK as isize),
                GetModuleHandleW(None).unwrap().into(),
                None,
            )
            .unwrap();

            let forward_btn = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("BUTTON"),
                w!("Forward →"),
                WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_PUSHBUTTON as u32),
                MARGIN + BUTTON_WIDTH + MARGIN,
                MARGIN,
                BUTTON_WIDTH,
                BUTTON_HEIGHT,
                hwnd,
                HMENU(ID_FORWARD as isize),
                GetModuleHandleW(None).unwrap().into(),
                None,
            )
            .unwrap();

            let refresh_btn = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("BUTTON"),
                w!("⟳ Refresh"),
                WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_PUSHBUTTON as u32),
                MARGIN + (BUTTON_WIDTH + MARGIN) * 2,
                MARGIN,
                BUTTON_WIDTH,
                BUTTON_HEIGHT,
                hwnd,
                HMENU(ID_REFRESH as isize),
                GetModuleHandleW(None).unwrap().into(),
                None,
            )
            .unwrap();

            // Create URL bar
            let url_bar = CreateWindowExW(
                WS_EX_CLIENTEDGE,
                w!("EDIT"),
                w!("https://www.google.com"),
                WS_CHILD | WS_VISIBLE | WINDOW_STYLE(ES_LEFT as u32 | ES_AUTOHSCROLL as u32),
                MARGIN + (BUTTON_WIDTH + MARGIN) * 3,
                MARGIN,
                WINDOW_WIDTH - (MARGIN + (BUTTON_WIDTH + MARGIN) * 3) - BUTTON_WIDTH - MARGIN * 2,
                BUTTON_HEIGHT,
                hwnd,
                HMENU(ID_URL_BAR as isize),
                GetModuleHandleW(None).unwrap().into(),
                None,
            )
            .unwrap();

            // Create Go button
            let go_btn = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("BUTTON"),
                w!("Go"),
                WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_PUSHBUTTON as u32),
                WINDOW_WIDTH - BUTTON_WIDTH - MARGIN,
                MARGIN,
                BUTTON_WIDTH,
                BUTTON_HEIGHT,
                hwnd,
                HMENU(ID_GO as isize),
                GetModuleHandleW(None).unwrap().into(),
                None,
            )
            .unwrap();

            // Store browser state
            let state = Box::new(Arc::new(Mutex::new(BrowserState::new())));
            let state_ptr = Box::into_raw(state);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize);

            // Store URL bar handle
            if let Ok(state_arc) = (*(state_ptr as *mut Arc<Mutex<BrowserState>>)).lock() {
                let mut state_guard = state_arc;
                // This won't work, we need to restructure
            }

            // Initialize WebView2
            let state_clone = Arc::clone(&*(state_ptr as *mut Arc<Mutex<BrowserState>>));
            let hwnd_clone = hwnd;

            std::thread::spawn(move || {
                let _ = initialize_webview(hwnd_clone, state_clone, url_bar);
            });

            LRESULT(0)
        }
        WM_SIZE => {
            let width = (lparam.0 & 0xFFFF) as i32;
            let height = ((lparam.0 >> 16) & 0xFFFF) as i32;

            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Arc<Mutex<BrowserState>>;
            if !state_ptr.is_null() {
                if let Ok(state) = (*state_ptr).lock() {
                    if let Some(controller) = &state.webview_controller {
                        let mut bounds = RECT {
                            left: 0,
                            top: URL_BAR_HEIGHT + MARGIN * 2,
                            right: width,
                            bottom: height,
                        };
                        let _ = controller.SetBounds(bounds);
                    }
                }
            }

            LRESULT(0)
        }
        WM_COMMAND => {
            let command_id = (wparam.0 & 0xFFFF) as u16;
            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Arc<Mutex<BrowserState>>;

            if !state_ptr.is_null() {
                if let Ok(state) = (*state_ptr).lock() {
                    match command_id {
                        ID_BACK => {
                            if let Some(webview) = &state.webview {
                                let _ = webview.GoBack();
                            }
                        }
                        ID_FORWARD => {
                            if let Some(webview) = &state.webview {
                                let _ = webview.GoForward();
                            }
                        }
                        ID_REFRESH => {
                            if let Some(webview) = &state.webview {
                                let _ = webview.Reload();
                            }
                        }
                        ID_GO => {
                            navigate_to_url(&state);
                        }
                        _ => {}
                    }
                }
            }

            LRESULT(0)
        }
        WM_DESTROY => {
            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Arc<Mutex<BrowserState>>;
            if !state_ptr.is_null() {
                let _ = Box::from_raw(state_ptr);
            }
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn navigate_to_url(state: &BrowserState) {
    if let Some(webview) = &state.webview {
        let mut buffer = [0u16; 2048];
        let len = GetWindowTextW(state.url_bar, &mut buffer);
        if len > 0 {
            let url = String::from_utf16_lossy(&buffer[..len as usize]);
            let url_with_protocol = if !url.starts_with("http://") && !url.starts_with("https://") {
                format!("https://{}", url)
            } else {
                url
            };
            let _ = webview.Navigate(&HSTRING::from(url_with_protocol));
        }
    }
}

fn initialize_webview(
    hwnd: HWND,
    state: Arc<Mutex<BrowserState>>,
    url_bar: HWND,
) -> Result<()> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED)?;

        let environment_completed = Box::new(EnvironmentCompletedHandler::new(hwnd, state, url_bar));
        let handler: ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler =
            environment_completed.into();

        CreateCoreWebView2EnvironmentWithOptions(
            None,
            None,
            None,
            &handler,
        )?;

        Ok(())
    }
}

#[windows::core::implement(ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler)]
struct EnvironmentCompletedHandler {
    hwnd: HWND,
    state: Arc<Mutex<BrowserState>>,
    url_bar: HWND,
}

impl EnvironmentCompletedHandler {
    fn new(hwnd: HWND, state: Arc<Mutex<BrowserState>>, url_bar: HWND) -> Self {
        Self { hwnd, state, url_bar }
    }
}

impl ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler_Impl
    for EnvironmentCompletedHandler_Impl
{
    fn Invoke(
        &self,
        _result: HRESULT,
        environment: Option<&ICoreWebView2Environment>,
    ) -> Result<()> {
        if let Some(env) = environment {
            let controller_completed =
                Box::new(ControllerCompletedHandler::new(self.hwnd, self.state.clone(), self.url_bar));
            let handler: ICoreWebView2CreateCoreWebView2ControllerCompletedHandler =
                controller_completed.into();

            unsafe {
                env.CreateCoreWebView2Controller(self.hwnd, &handler)?;
            }
        }
        Ok(())
    }
}

#[windows::core::implement(ICoreWebView2CreateCoreWebView2ControllerCompletedHandler)]
struct ControllerCompletedHandler {
    hwnd: HWND,
    state: Arc<Mutex<BrowserState>>,
    url_bar: HWND,
}

impl ControllerCompletedHandler {
    fn new(hwnd: HWND, state: Arc<Mutex<BrowserState>>, url_bar: HWND) -> Self {
        Self { hwnd, state, url_bar }
    }
}

impl ICoreWebView2CreateCoreWebView2ControllerCompletedHandler_Impl
    for ControllerCompletedHandler_Impl
{
    fn Invoke(
        &self,
        _result: HRESULT,
        controller: Option<&ICoreWebView2Controller>,
    ) -> Result<()> {
        if let Some(ctrl) = controller {
            unsafe {
                let webview = ctrl.CoreWebView2()?;

                // Set initial bounds
                let mut rect = RECT::default();
                GetClientRect(self.hwnd, &mut rect)?;
                rect.top = URL_BAR_HEIGHT + MARGIN * 2;

                ctrl.SetBounds(rect)?;
                ctrl.SetIsVisible(true)?;

                // Navigate to initial URL
                webview.Navigate(w!("https://www.google.com"))?;

                // Store in state
                if let Ok(mut state) = self.state.lock() {
                    state.webview_controller = Some(ctrl.clone());
                    state.webview = Some(webview.clone());
                    state.url_bar = self.url_bar;
                }

                // Add navigation completed handler to update URL bar
                let url_bar = self.url_bar;
                let nav_handler = Box::new(NavigationCompletedHandler::new(url_bar));
                let handler: ICoreWebView2NavigationCompletedEventHandler = nav_handler.into();
                webview.add_NavigationCompleted(&handler)?;
            }
        }
        Ok(())
    }
}

#[windows::core::implement(ICoreWebView2NavigationCompletedEventHandler)]
struct NavigationCompletedHandler {
    url_bar: HWND,
}

impl NavigationCompletedHandler {
    fn new(url_bar: HWND) -> Self {
        Self { url_bar }
    }
}

impl ICoreWebView2NavigationCompletedEventHandler_Impl for NavigationCompletedHandler_Impl {
    fn Invoke(
        &self,
        sender: Option<&ICoreWebView2>,
        _args: Option<&ICoreWebView2NavigationCompletedEventArgs>,
    ) -> Result<()> {
        if let Some(webview) = sender {
            unsafe {
                let url = webview.Source()?;
                SetWindowTextW(self.url_bar, &url)?;
            }
        }
        Ok(())
    }
}
