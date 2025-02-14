//! Safe Rust bindings for the Luigi UI library.
//!
//! This library provides a safe wrapper around the native C Luigi UI library,
//! offering an idiomatic Rust interface while maintaining all the original functionality.

mod sys;

use std::ffi::{c_void, CString};
use std::ptr;

// Re-export common constants
pub use sys::{
    UI_ALIGN_CENTER, UI_ALIGN_LEFT, UI_ALIGN_RIGHT, UI_BUTTON_CAN_FOCUS, UI_BUTTON_CHECKED,
    UI_BUTTON_DROP_DOWN, UI_BUTTON_MENU_ITEM, UI_BUTTON_SMALL, UI_ELEMENT_H_FILL,
    UI_ELEMENT_PARENT_PUSH, UI_ELEMENT_V_FILL, UI_PANEL_BORDER, UI_PANEL_EXPAND, UI_PANEL_GRAY,
    UI_PANEL_HORIZONTAL, UI_PANEL_MEDIUM_SPACING, UI_PANEL_SCROLL, UI_PANEL_SMALL_SPACING,
    UI_PANEL_WHITE, UI_WINDOW_CENTER_IN_OWNER, UI_WINDOW_INSPECTOR, UI_WINDOW_MAXIMIZE,
    UI_WINDOW_MENU,
};

// Define message constants directly since they don't exist in sys
pub const UI_MSG_TABLE_GET_ITEM: i32 = 51; // These values should match the C enum
pub const UI_MSG_LEFT_DOWN: i32 = 11; // These values should match the C enum

/// Error types that can occur in Luigi operations
#[derive(Debug)]
pub enum Error {
    /// A null pointer was encountered where a valid pointer was expected
    NullPointer,
    /// Failed to convert a string to a C-compatible format
    InvalidString,
    /// Failed to create a UI element
    CreateFailed,
}

/// Result type for Luigi operations
pub type Result<T> = std::result::Result<T, Error>;

/// Common trait implemented by all UI elements
pub trait Element {
    /// Get the raw pointer to the underlying UIElement
    fn raw_element(&self) -> *mut sys::UIElement;

    /// Destroy this element and remove it from the UI hierarchy
    fn destroy(&mut self) {
        unsafe { sys::UIElementDestroy(self.raw_element()) }
    }

    /// Refresh this element's layout and appearance
    fn refresh(&mut self) {
        unsafe { sys::UIElementRefresh(self.raw_element()) }
    }
}

/// Handler for UI element events
pub trait EventHandler {
    fn handle(&self, element: &mut dyn Element, message: i32, data: &str) -> String;
}

/// A top-level window containing UI elements
pub struct Window {
    raw: *mut sys::UIWindow,
}

impl Window {
    /// Create a new window with the given title, dimensions and flags
    ///
    /// # Arguments
    /// * `title` - The window title
    /// * `width` - Window width in pixels (0 for default)
    /// * `height` - Window height in pixels (0 for default)
    /// * `flags` - Window creation flags
    pub fn new(title: &str, width: i32, height: i32, flags: u32) -> Result<Self> {
        let title = CString::new(title).map_err(|_| Error::InvalidString)?;
        let raw =
            unsafe { sys::UIWindowCreate(ptr::null_mut(), flags, title.as_ptr(), width, height) };
        if raw.is_null() {
            return Err(Error::CreateFailed);
        }
        Ok(Self { raw })
    }

    /// Register a keyboard shortcut for this window
    ///
    /// # Arguments
    /// * `shortcut` - The shortcut to register
    pub fn register_shortcut(&mut self, shortcut: Shortcut) {
        unsafe {
            sys::UIWindowRegisterShortcut(self.raw, shortcut.into_raw());
        }
    }
}

// Add Element trait implementation for Window
impl Element for Window {
    fn raw_element(&self) -> *mut sys::UIElement {
        unsafe { &mut (*self.raw).e as *mut sys::UIElement }
    }
}

/// A clickable button element
pub struct Button {
    raw: *mut sys::UIButton,
}

impl Button {
    /// Create a new button with the given label and flags
    ///
    /// # Arguments
    /// * `parent` - Parent element to attach this button to
    /// * `flags` - Button creation flags
    /// * `label` - Text label for the button
    pub fn new(parent: &impl Element, flags: u32, label: &str) -> Result<Self> {
        let label = CString::new(label).map_err(|_| Error::InvalidString)?;
        let raw = unsafe { sys::UIButtonCreate(parent.raw_element(), flags, label.as_ptr(), -1) };
        if raw.is_null() {
            return Err(Error::CreateFailed);
        }
        Ok(Self { raw })
    }

    pub fn invoke(&self, callback: Box<dyn Fn()>) {
        unsafe {
            let raw = self.raw;
            // Store callback in a Box that won't be dropped
            let callback_box = Box::new(callback);
            (*raw).invoke = Some(Self::invoke_handler);
            (*raw).e.cp = Box::into_raw(callback_box) as *mut c_void;
        }
    }

    extern "C" fn invoke_handler(cp: *mut c_void) {
        unsafe {
            let callback = &*(cp as *const Box<dyn Fn()>);
            callback();
        }
    }
}

impl Element for Button {
    fn raw_element(&self) -> *mut sys::UIElement {
        unsafe { &mut (*self.raw).e }
    }
}

/// A panel container element that can hold other elements
pub struct Panel {
    raw: *mut sys::UIPanel,
}

impl Panel {
    /// Create a new panel with the specified flags
    ///
    /// # Arguments
    /// * `parent` - Parent element to attach this panel to
    /// * `flags` - Panel creation flags
    pub fn new(parent: &impl Element, flags: u32) -> Result<Self> {
        let raw = unsafe { sys::UIPanelCreate(parent.raw_element(), flags) };
        if raw.is_null() {
            return Err(Error::CreateFailed);
        }
        Ok(Self { raw })
    }
}

impl Element for Panel {
    fn raw_element(&self) -> *mut sys::UIElement {
        unsafe { &mut (*self.raw).e }
    }
}

/// A keyboard shortcut definition
pub struct Shortcut {
    code: isize,
    ctrl: bool,
    shift: bool,
    alt: bool,
    invoke: Box<dyn Fn()>,
}

impl Shortcut {
    /// Create a new keyboard shortcut
    ///
    /// # Arguments
    /// * `code` - Key code for the shortcut
    /// * `ctrl` - Whether Control key is required
    /// * `shift` - Whether Shift key is required
    /// * `alt` - Whether Alt key is required
    /// * `invoke` - Callback function to execute when shortcut is triggered
    pub fn new(code: i32, ctrl: bool, shift: bool, alt: bool, invoke: impl Fn() + 'static) -> Self {
        Self {
            code: code as isize,
            ctrl,
            shift,
            alt,
            invoke: Box::new(invoke),
        }
    }

    unsafe fn into_raw(self) -> sys::UIShortcut {
        extern "C" fn trampoline(data: *mut c_void) {
            let closure = unsafe { &*(data as *const Box<dyn Fn()>) };
            closure();
        }

        sys::UIShortcut {
            code: self.code,
            ctrl: self.ctrl,
            shift: self.shift,
            alt: self.alt,
            invoke: Some(trampoline),
            cp: Box::into_raw(Box::new(self.invoke)) as *mut c_void,
        }
    }
}

/// Initialize the Luigi UI system.
/// Must be called before creating any windows or UI elements.
pub fn init() {
    unsafe {
        sys::UIInitialise();
        // Use an explicit font ("Arial") instead of null
        let font_name = CString::new("Arial").unwrap();
        let font = sys::UIFontCreate(font_name.as_ptr(), 16);
        sys::UIFontActivate(font);
    };
}

/// Start the UI message loop.
/// This function blocks until the application exits.
pub fn message_loop() -> i32 {
    unsafe { sys::UIMessageLoop() }
}

/// Create a rectangle with the given coordinates
///
/// # Arguments
/// * `l` - Left coordinate
/// * `r` - Right coordinate
/// * `t` - Top coordinate
/// * `b` - Bottom coordinate
pub fn rect(l: i32, r: i32, t: i32, b: i32) -> sys::UIRectangle {
    sys::UIRectangle { l, r, t, b }
}

/// Add two rectangles together
pub fn rect_add(a: sys::UIRectangle, b: sys::UIRectangle) -> sys::UIRectangle {
    unsafe { sys::UIRectangleAdd(a, b) }
}

/// Get the intersection of two rectangles
pub fn rect_intersect(a: sys::UIRectangle, b: sys::UIRectangle) -> sys::UIRectangle {
    unsafe { sys::UIRectangleIntersection(a, b) }
}

/// Convert an RGB color value to HSV color space
///
/// # Arguments
/// * `rgb` - RGB color value as a u32
///
/// # Returns
/// * `Some((h,s,v))` - HSV values if conversion succeeded
/// * `None` - If conversion failed
pub fn color_to_hsv(rgb: u32) -> Option<(f32, f32, f32)> {
    let mut h = 0.0;
    let mut s = 0.0;
    let mut v = 0.0;
    let result = unsafe { sys::UIColorToHSV(rgb, &mut h, &mut s, &mut v) };
    if result {
        Some((h, s, v))
    } else {
        None
    }
}

/// Convert HSV color values to RGB color space
///
/// # Arguments
/// * `h` - Hue value (0.0-1.0)
/// * `s` - Saturation value (0.0-1.0)
/// * `v` - Value/brightness (0.0-1.0)
pub fn color_to_rgb(h: f32, s: f32, v: f32) -> u32 {
    let mut rgb = 0;
    unsafe { sys::UIColorToRGB(h, s, v, &mut rgb) };
    rgb
}

/// Measure the width of a string in pixels
pub fn measure_string_width(text: &str) -> i32 {
    let text = CString::new(text).unwrap_or_default();
    unsafe { sys::UIMeasureStringWidth(text.as_ptr(), -1) }
}

/// Get the standard height of a line of text in pixels
pub fn measure_string_height() -> i32 {
    unsafe { sys::UIMeasureStringHeight() }
}

/// Get the current animation clock value in milliseconds
pub fn animate_clock() -> u64 {
    unsafe { sys::UIAnimateClock() }
}

// Add new widget types
pub struct Label {
    raw: *mut sys::UILabel,
}

impl Label {
    pub fn new(parent: &impl Element, flags: u32, text: &str) -> Result<Self> {
        let text = CString::new(text).map_err(|_| Error::InvalidString)?;
        let raw = unsafe { sys::UILabelCreate(parent.raw_element(), flags, text.as_ptr(), -1) };
        if raw.is_null() {
            return Err(Error::CreateFailed);
        }
        Ok(Self { raw })
    }

    pub fn set_content(&self, text: &str) {
        let text = CString::new(text)
            .map_err(|_| Error::InvalidString)
            .unwrap_or_default();
        unsafe {
            // Changed -1 to text.as_bytes().len() as i32 to properly handle string length
            sys::UILabelSetContent(self.raw, text.as_ptr(), text.as_bytes().len() as isize)
        };
    }
}

impl Element for Label {
    fn raw_element(&self) -> *mut sys::UIElement {
        unsafe { &mut (*self.raw).e }
    }
}

pub struct Table {
    raw: *mut sys::UITable,
}

impl Table {
    pub fn new(parent: &impl Element, flags: u32, columns: &str) -> Result<Self> {
        let columns = CString::new(columns).map_err(|_| Error::InvalidString)?;
        let raw = unsafe { sys::UITableCreate(parent.raw_element(), flags, columns.as_ptr()) };
        if raw.is_null() {
            return Err(Error::CreateFailed);
        }
        Ok(Self { raw })
    }

    pub fn set_item_count(&self, count: usize) {
        unsafe { (*self.raw).itemCount = count as i32 };
    }

    pub fn set_handler(&self, handler: Box<dyn EventHandler>) {
        unsafe {
            let raw = self.raw_element();
            (*raw).cp = Box::into_raw(handler) as *mut c_void;
            #[cfg(target_os = "linux")]
            {
                (*raw).messageUser = Some(Self::message_handler as unsafe extern "C" fn(*mut sys::UIElement, u32, i32, *mut c_void) -> i32);
            }
            #[cfg(not(target_os = "linux"))]
            {
                (*raw).messageUser = Some(Self::message_handler as unsafe extern "C" fn(*mut sys::UIElement, i32, i32, *mut c_void) -> i32);
            }
        }
    }

    #[cfg(target_os = "linux")]
    extern "C" fn message_handler(
        element: *mut sys::UIElement,
        message: u32,
        di: i32,
        dp: *mut c_void,
    ) -> i32 {
        unsafe {
            let handler = &*((*element).cp as *const Box<dyn EventHandler>);
            let mut wrapper = ElementWrapper { raw: element };
            let data = if dp.is_null() { "" } else {
                std::ffi::CStr::from_ptr(dp as *const i8).to_str().unwrap_or("")
            };
            let result = handler.handle(&mut wrapper, message as i32, data);
            if !result.is_empty() {
                if let Some(buffer) = dp.cast::<sys::UITableGetItem>().as_mut() {
                    let bytes = buffer.bufferBytes.min(result.len());
                    std::ptr::copy_nonoverlapping(result.as_ptr(), buffer.buffer as *mut u8, bytes);
                    return bytes as i32;
                }
            }
            0
        }
    }

    #[cfg(not(target_os = "linux"))]
    extern "C" fn message_handler(
        element: *mut sys::UIElement,
        message: i32,
        _di: i32,
        dp: *mut c_void,
    ) -> i32 {
        unsafe {
            let handler = &*((*element).cp as *const Box<dyn EventHandler>);
            let mut wrapper = ElementWrapper { raw: element };
            let data = if dp.is_null() { "" } else {
                std::ffi::CStr::from_ptr(dp as *const i8).to_str().unwrap_or("")
            };
            let result = handler.handle(&mut wrapper, message, data);
            if !result.is_empty() {
                if let Some(buffer) = dp.cast::<sys::UITableGetItem>().as_mut() {
                    let bytes = buffer.bufferBytes.min(result.len());
                    std::ptr::copy_nonoverlapping(result.as_ptr(), buffer.buffer as *mut u8, bytes);
                    return bytes as i32;
                }
            }
            0
        }
    }
}

impl Element for Table {
    fn raw_element(&self) -> *mut sys::UIElement {
        unsafe { &mut (*self.raw).e }
    }
}

// Helper wrapper to implement Element trait
struct ElementWrapper {
    raw: *mut sys::UIElement,
}

impl Element for ElementWrapper {
    fn raw_element(&self) -> *mut sys::UIElement {
        self.raw
    }
}

pub struct TextBox {
    raw: *mut sys::UITextbox,
}

impl TextBox {
    pub fn new(parent: &impl Element, flags: u32) -> Result<Self> {
        let raw = unsafe { sys::UITextboxCreate(parent.raw_element(), flags) };
        if raw.is_null() {
            return Err(Error::CreateFailed);
        }
        Ok(Self { raw })
    }

    pub fn get_text(&self) -> String {
        unsafe {
            let text = std::slice::from_raw_parts(
                (*self.raw).string as *const u8,
                (*self.raw).bytes as usize,
            );
            String::from_utf8_lossy(text).to_string()
        }
    }

    pub fn is_empty(&self) -> bool {
        unsafe { (*self.raw).bytes == 0 }
    }
}

impl Element for TextBox {
    fn raw_element(&self) -> *mut sys::UIElement {
        unsafe { &mut (*self.raw).e }
    }
}

// Additional Widgets

pub struct Checkbox {
    raw: *mut sys::UICheckbox,
}

impl Checkbox {
    pub fn new(parent: &impl Element, flags: u32, label: &str) -> Result<Self> {
        let label = CString::new(label).map_err(|_| Error::InvalidString)?;
        let raw = unsafe { sys::UICheckboxCreate(parent.raw_element(), flags, label.as_ptr(), -1) };
        if raw.is_null() {
            return Err(Error::CreateFailed);
        }
        Ok(Self { raw })
    }

    pub fn get_check_state(&self) -> u8 {
        unsafe { (*self.raw).check }
    }
}

impl Element for Checkbox {
    fn raw_element(&self) -> *mut sys::UIElement {
        unsafe { &mut (*self.raw).e }
    }
}

pub struct Code {
    raw: *mut sys::UICode,
}

impl Code {
    pub fn new(parent: &impl Element, flags: u32) -> Result<Self> {
        let raw = unsafe { sys::UICodeCreate(parent.raw_element(), flags) };
        if raw.is_null() {
            return Err(Error::CreateFailed);
        }
        Ok(Self { raw })
    }

    pub fn insert_content(&mut self, content: &str, replace: bool) {
        unsafe {
            sys::UICodeInsertContent(
                self.raw,
                content.as_ptr() as *const i8,
                content.len() as isize,
                replace,
            )
        }
    }

    pub fn focus_line(&mut self, line: i32) {
        unsafe { sys::UICodeFocusLine(self.raw, line) }
    }
}

impl Element for Code {
    fn raw_element(&self) -> *mut sys::UIElement {
        unsafe { &mut (*self.raw).e }
    }
}

pub struct Gauge {
    raw: *mut sys::UIGauge,
}

impl Gauge {
    pub fn new(parent: &impl Element, flags: u32) -> Result<Self> {
        let raw = unsafe { sys::UIGaugeCreate(parent.raw_element(), flags) };
        if raw.is_null() {
            return Err(Error::CreateFailed);
        }
        Ok(Self { raw })
    }

    pub fn set_position(&mut self, position: f32) {
        unsafe { (*self.raw).position = position }
    }
}

impl Element for Gauge {
    fn raw_element(&self) -> *mut sys::UIElement {
        unsafe { &mut (*self.raw).e }
    }
}

pub struct Slider {
    raw: *mut sys::UISlider,
}

impl Slider {
    pub fn new(parent: &impl Element, flags: u32) -> Result<Self> {
        let raw = unsafe { sys::UISliderCreate(parent.raw_element(), flags) };
        if raw.is_null() {
            return Err(Error::CreateFailed);
        }
        Ok(Self { raw })
    }

    pub fn set_position(&mut self, position: f32) {
        unsafe { (*self.raw).position = position }
    }

    pub fn set_steps(&mut self, steps: i32) {
        unsafe { (*self.raw).steps = steps }
    }
}

impl Element for Slider {
    fn raw_element(&self) -> *mut sys::UIElement {
        unsafe { &mut (*self.raw).e }
    }
}

pub struct MDIClient {
    raw: *mut sys::UIMDIClient,
}

impl MDIClient {
    pub fn new(parent: &impl Element, flags: u32) -> Result<Self> {
        let raw = unsafe { sys::UIMDIClientCreate(parent.raw_element(), flags) };
        if raw.is_null() {
            return Err(Error::CreateFailed);
        }
        Ok(Self { raw })
    }
}

impl Element for MDIClient {
    fn raw_element(&self) -> *mut sys::UIElement {
        unsafe { &mut (*self.raw).e }
    }
}

pub struct MDIChild {
    raw: *mut sys::UIMDIChild,
}

impl MDIChild {
    pub fn new(
        parent: &impl Element,
        flags: u32,
        bounds: sys::UIRectangle,
        title: &str,
    ) -> Result<Self> {
        let title = CString::new(title).map_err(|_| Error::InvalidString)?;
        let raw = unsafe {
            sys::UIMDIChildCreate(parent.raw_element(), flags, bounds, title.as_ptr(), -1)
        };
        if raw.is_null() {
            return Err(Error::CreateFailed);
        }
        Ok(Self { raw })
    }
}

impl Element for MDIChild {
    fn raw_element(&self) -> *mut sys::UIElement {
        unsafe { &mut (*self.raw).e }
    }
}

pub struct Menu {
    raw: *mut sys::UIMenu,
}

impl Menu {
    pub fn new(parent: &impl Element, flags: u32) -> Result<Self> {
        let raw = unsafe { sys::UIMenuCreate(parent.raw_element(), flags) };
        if raw.is_null() {
            return Err(Error::CreateFailed);
        }
        Ok(Self { raw })
    }

    pub fn add_item(&mut self, flags: u32, label: &str, callback: Box<dyn Fn()>) {
        let label = CString::new(label).unwrap_or_default();
        extern "C" fn trampoline(data: *mut c_void) {
            let closure = unsafe { &*(data as *const Box<dyn Fn()>) };
            closure();
        }
        let cp = Box::into_raw(callback) as *mut c_void;
        unsafe {
            sys::UIMenuAddItem(self.raw, flags, label.as_ptr(), -1, Some(trampoline), cp);
        }
    }

    pub fn show(&self) {
        unsafe { sys::UIMenuShow(self.raw) }
    }
}

impl Element for Menu {
    fn raw_element(&self) -> *mut sys::UIElement {
        unsafe { &mut (*self.raw).e }
    }
}

pub struct ColorPicker {
    raw: *mut sys::UIColorPicker,
}

impl ColorPicker {
    pub fn new(parent: &impl Element, flags: u32) -> Result<Self> {
        let raw = unsafe { sys::UIColorPickerCreate(parent.raw_element(), flags) };
        if raw.is_null() {
            return Err(Error::CreateFailed);
        }
        Ok(Self { raw })
    }

    pub fn get_color(&self) -> (f32, f32, f32, f32) {
        unsafe {
            (
                (*self.raw).hue,
                (*self.raw).saturation,
                (*self.raw).value,
                (*self.raw).opacity,
            )
        }
    }

    pub fn set_color(&mut self, h: f32, s: f32, v: f32, o: f32) {
        unsafe {
            (*self.raw).hue = h;
            (*self.raw).saturation = s;
            (*self.raw).value = v;
            (*self.raw).opacity = o;
        }
    }
}

impl Element for ColorPicker {
    fn raw_element(&self) -> *mut sys::UIElement {
        unsafe { &mut (*self.raw).e }
    }
}

pub struct ImageDisplay {
    raw: *mut sys::UIImageDisplay,
}

impl ImageDisplay {
    pub fn new(
        parent: &impl Element,
        flags: u32,
        bits: &[u32],
        width: usize,
        height: usize,
    ) -> Result<Self> {
        let raw = unsafe {
            sys::UIImageDisplayCreate(
                parent.raw_element(),
                flags,
                bits.as_ptr() as *mut u32, // Cast to mutable pointer
                width,
                height,
                width * 4,
            )
        };
        if raw.is_null() {
            return Err(Error::CreateFailed);
        }
        Ok(Self { raw })
    }

    pub fn set_content(&mut self, bits: &[u32], width: usize, height: usize) {
        unsafe {
            sys::UIImageDisplaySetContent(
                self.raw,
                bits.as_ptr() as *mut u32, // Cast to mutable pointer
                width,
                height,
                width * 4,
            )
        }
    }
}

impl Element for ImageDisplay {
    fn raw_element(&self) -> *mut sys::UIElement {
        unsafe { &mut (*self.raw).e }
    }
}
