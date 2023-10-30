use cstr_core::CString;

/*
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
*/

use lvgl;
use lvgl::style::Style;
use lvgl::widgets::{Arc, Label};
use lvgl::{Align, Color, Display, DrawBuffer, LvError, Part, Screen, Widget};
use lvgl_sys;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

use std::cell::RefCell;

use core::pin::Pin;

fn mem_info() -> lvgl_sys::lv_mem_monitor_t {
    let mut info = lvgl_sys::lv_mem_monitor_t {
        total_size: 0,
        free_cnt: 0,
        free_size: 0,
        free_biggest_size: 0,
        used_cnt: 0,
        max_used: 0,
        used_pct: 0,
        frag_pct: 0,
    };
    unsafe {
        lvgl_sys::lv_mem_monitor(&mut info as *mut _);
    }
    info
}

struct DisplayHandle<'a> {
    pub display: Pin<Box<Display>>,
    pub arc: RefCell<Arc<'a>>,
}

fn mk_display() -> Result< Pin<Box<DisplayHandle<'static>> >, LvError> {
    const HOR_RES: u32 = 800;
    const VER_RES: u32 = 480;
    const DISP_BUF_SIZE: u32 = 131072; //128*1024

    println!("meminfo init: {:?}", mem_info());

    /*LittlevGL init*/
    lvgl::init();
    /*Linux frame buffer device init*/
    unsafe {
        lvgl_sys::fbdev_init();
    }

    /*Initialize a descriptor for the buffer*/
    let buffer = DrawBuffer::<{ (DISP_BUF_SIZE) as usize }>::default();

    /*Initialize and register a display driver*/
    let mut display = Box::pin(unsafe {
        Display::register_raw(
            buffer,
            HOR_RES,
            VER_RES,
            Some(lvgl_sys::fbdev_flush),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(lvgl_sys::fbdev_exit),
        )?
    });

    let mut screen = display.get_scr_act()?;

    let mut screen_style = Box::new(Style::default());
    screen_style.set_bg_color(Color::from_rgb((255, 255, 255)));
    screen_style.set_radius(0);
    screen.add_style(Part::Main, &mut *screen_style);


    // Create the arc object
    let mut arc = Arc::create(&mut screen)?;
    arc.set_size(150, 150);
    arc.set_align(Align::Center, 0, 10);
    arc.set_start_angle(135)?;
    arc.set_end_angle(135)?;

    let mut loading_lbl = Label::create(&mut screen)?;
    loading_lbl.set_text(CString::new("Loading...").unwrap().as_c_str())?;
    loading_lbl.set_align(Align::OutTopMid, 0, 0);

    let mut loading_style = Style::default();
    loading_style.set_text_color(Color::from_rgb((0, 0, 0)));
    loading_lbl.add_style(Part::Main, &mut loading_style);

    let handle = Box::pin(DisplayHandle {
        display,
        arc: RefCell::new(arc),
    });
    //let handle = Box::leak(handle);
    //display_update(handle)?;

    Ok(handle)
}

fn display_update(handle:Pin<Box<DisplayHandle>>) -> Result<(), LvError> {
    let mut angle = 0;
    let mut forward = true;
    let mut i = 0;

    // retrieve mutable arc handle from display handle
    let mut arc = handle.arc.borrow_mut();
    //let mut window= handle.window.borrow_mut();

    'running: loop {
        let start = Instant::now();
        if i > 270 {
            forward = if forward { false } else { true };
            i = 1;
            println!("mem info running: {:?}", mem_info());
        }
        angle = if forward { angle + 1 } else { angle - 1 };

        arc.set_end_angle(angle + 135)?;
        println!("arc.set_end_angle");
        i += 1;
        println!("i");

        lvgl::task_handler();
        println!("lvgl::task_handler");

        sleep(Duration::from_millis(15));
        println!("Duration::from_millis");
        lvgl::tick_inc(Instant::now().duration_since(start));

    }
}

fn main() -> Result<(), LvError> {
    let handle = mk_display()?;

    println!("mk_display");
    // handle.update should be callable from an asynchronous function (i.e. a system timer)
    display_update(handle)?;

    println!("display_update");
    Ok(())
}
