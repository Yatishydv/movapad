//! macOS mouse controller using Core Graphics.
//!
//! Uses CGEventCreateMouseEvent / CGEventPost for cursor movement and clicks,
//! and CGEventCreateScrollWheelEvent2 for scrolling.
//!
//! Requires Accessibility permission (AXIsProcessTrusted).

use core_graphics::display::CGDisplay;
use core_graphics::event::{
    CGEvent, CGEventTapLocation, CGEventType, CGMouseButton, ScrollEventUnit,
};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use core_graphics::geometry::CGPoint;

use super::{InputError, InputResult, MouseController};

/// macOS implementation of MouseController using Core Graphics APIs.
pub struct MacOSMouseController {
    /// Cached event source to avoid repeated allocation.
    source_state: CGEventSourceStateID,
}

impl MacOSMouseController {
    pub fn new() -> Self {
        Self {
            source_state: CGEventSourceStateID::HIDSystemState,
        }
    }

    /// Get the current mouse position.
    fn current_position(&self) -> CGPoint {
        let source = CGEventSource::new(self.source_state).unwrap();
        let event = CGEvent::new(source);
        match event {
            Ok(e) => e.location(),
            Err(_) => {
                // Fallback: get from display
                let display = CGDisplay::main();
                let bounds = display.bounds();
                CGPoint::new(bounds.size.width / 2.0, bounds.size.height / 2.0)
            }
        }
    }

    /// Post a mouse event at the given position.
    fn post_mouse_event(
        &self,
        event_type: CGEventType,
        position: CGPoint,
        button: CGMouseButton,
    ) -> InputResult<()> {
        let source = CGEventSource::new(self.source_state)
            .map_err(|_| InputError::PlatformError("failed to create event source".into()))?;

        let event = CGEvent::new_mouse_event(source, event_type, position, button)
            .map_err(|_| InputError::PlatformError("failed to create mouse event".into()))?;

        event.post(CGEventTapLocation::HID);
        Ok(())
    }
}

impl MouseController for MacOSMouseController {
    fn move_relative(&self, dx: i32, dy: i32) -> InputResult<()> {
        let current = self.current_position();
        let new_pos = CGPoint::new(
            current.x + dx as f64,
            current.y + dy as f64,
        );

        self.post_mouse_event(CGEventType::MouseMoved, new_pos, CGMouseButton::Left)
    }

    fn left_down(&self) -> InputResult<()> {
        let pos = self.current_position();
        self.post_mouse_event(CGEventType::LeftMouseDown, pos, CGMouseButton::Left)
    }

    fn left_up(&self) -> InputResult<()> {
        let pos = self.current_position();
        self.post_mouse_event(CGEventType::LeftMouseUp, pos, CGMouseButton::Left)
    }

    fn right_down(&self) -> InputResult<()> {
        let pos = self.current_position();
        self.post_mouse_event(CGEventType::RightMouseDown, pos, CGMouseButton::Right)
    }

    fn right_up(&self) -> InputResult<()> {
        let pos = self.current_position();
        self.post_mouse_event(CGEventType::RightMouseUp, pos, CGMouseButton::Right)
    }

    fn scroll(&self, dx: i32, dy: i32) -> InputResult<()> {
        let source = CGEventSource::new(self.source_state)
            .map_err(|_| InputError::PlatformError("failed to create event source".into()))?;

        // macOS scroll: positive = up/left, but our protocol uses positive = up
        // so dy maps directly. For horizontal: positive dx = scroll right = negative scroll value in CG.
        let event = CGEvent::new_scroll_event(
            source,
            ScrollEventUnit::PIXEL,
            2, // wheel_count: 2 = vertical + horizontal

            dy,
            -dx, // CG horizontal scroll is inverted relative to our convention
            0,
        )
        .map_err(|_| InputError::PlatformError("failed to create scroll event".into()))?;

        event.post(CGEventTapLocation::HID);
        Ok(())
    }

    fn reset(&self) -> InputResult<()> {
        // Release both mouse buttons to prevent stuck state
        let pos = self.current_position();
        let _ = self.post_mouse_event(CGEventType::LeftMouseUp, pos, CGMouseButton::Left);
        let _ = self.post_mouse_event(CGEventType::RightMouseUp, pos, CGMouseButton::Right);
        log::info!("Mouse state reset — all buttons released");
        Ok(())
    }

    fn check_permissions(&self) -> InputResult<bool> {
        // Check if Accessibility permission is granted using the
        // ApplicationServices framework
        let trusted = unsafe {
            // AXIsProcessTrusted() from ApplicationServices
            unsafe extern "C" {
                fn AXIsProcessTrusted() -> bool;
            }
            AXIsProcessTrusted()
        };
        Ok(trusted)
    }
}
