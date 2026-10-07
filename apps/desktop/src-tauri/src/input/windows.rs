//! Windows mouse controller using Win32 SendInput API.
//!
//! Uses MOUSEINPUT struct with SendInput for all mouse operations.
//! No special permissions required on Windows.

#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_HWHEEL, MOUSEEVENTF_LEFTDOWN,
    MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP,
    MOUSEEVENTF_WHEEL, MOUSEINPUT,
};

use super::{InputError, InputResult, MouseController};

/// Windows implementation of MouseController using SendInput.
pub struct WindowsMouseController;

impl WindowsMouseController {
    pub fn new() -> Self {
        Self
    }

    #[cfg(target_os = "windows")]
    fn send_mouse_input(&self, input: MOUSEINPUT) -> InputResult<()> {
        let mut inputs = [INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: input,
            },
        }];

        let result = unsafe { SendInput(&mut inputs, std::mem::size_of::<INPUT>() as i32) };

        if result == 0 {
            Err(InputError::PlatformError(
                "SendInput returned 0".into(),
            ))
        } else {
            Ok(())
        }
    }
}

impl MouseController for WindowsMouseController {
    fn move_relative(&self, dx: i32, dy: i32) -> InputResult<()> {
        #[cfg(target_os = "windows")]
        {
            self.send_mouse_input(MOUSEINPUT {
                dx,
                dy,
                mouseData: 0,
                dwFlags: MOUSEEVENTF_MOVE,
                time: 0,
                dwExtraInfo: 0,
            })
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (dx, dy);
            Err(InputError::PlatformError("not on Windows".into()))
        }
    }

    fn left_down(&self) -> InputResult<()> {
        #[cfg(target_os = "windows")]
        {
            self.send_mouse_input(MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: 0,
                dwFlags: MOUSEEVENTF_LEFTDOWN,
                time: 0,
                dwExtraInfo: 0,
            })
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(InputError::PlatformError("not on Windows".into()))
        }
    }

    fn left_up(&self) -> InputResult<()> {
        #[cfg(target_os = "windows")]
        {
            self.send_mouse_input(MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: 0,
                dwFlags: MOUSEEVENTF_LEFTUP,
                time: 0,
                dwExtraInfo: 0,
            })
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(InputError::PlatformError("not on Windows".into()))
        }
    }

    fn right_down(&self) -> InputResult<()> {
        #[cfg(target_os = "windows")]
        {
            self.send_mouse_input(MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: 0,
                dwFlags: MOUSEEVENTF_RIGHTDOWN,
                time: 0,
                dwExtraInfo: 0,
            })
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(InputError::PlatformError("not on Windows".into()))
        }
    }

    fn right_up(&self) -> InputResult<()> {
        #[cfg(target_os = "windows")]
        {
            self.send_mouse_input(MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: 0,
                dwFlags: MOUSEEVENTF_RIGHTUP,
                time: 0,
                dwExtraInfo: 0,
            })
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(InputError::PlatformError("not on Windows".into()))
        }
    }

    fn scroll(&self, dx: i32, dy: i32) -> InputResult<()> {
        #[cfg(target_os = "windows")]
        {
            // Vertical scroll: WHEEL_DELTA = 120 per notch
            if dy != 0 {
                self.send_mouse_input(MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: dy as u32,
                    dwFlags: MOUSEEVENTF_WHEEL,
                    time: 0,
                    dwExtraInfo: 0,
                })?;
            }
            // Horizontal scroll
            if dx != 0 {
                self.send_mouse_input(MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: dx as u32,
                    dwFlags: MOUSEEVENTF_HWHEEL,
                    time: 0,
                    dwExtraInfo: 0,
                })?;
            }
            Ok(())
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (dx, dy);
            Err(InputError::PlatformError("not on Windows".into()))
        }
    }

    fn reset(&self) -> InputResult<()> {
        let _ = self.left_up();
        let _ = self.right_up();
        Ok(())
    }

    fn check_permissions(&self) -> InputResult<bool> {
        // Windows does not require special permissions for SendInput
        Ok(true)
    }
}
