#[macro_use]
extern crate objc;

use objc::rc::StrongPtr;
use objc::runtime::{Object};
use clap::{Arg, Command};
use cocoa::appkit::NSScreen;
use cocoa::base::{id, nil};
use cocoa::foundation::{NSArray, NSDictionary, NSString, NSURL};
use directories::{ ProjectDirs};
use serde::{Serialize, Deserialize};

fn nsstring(s: &str) -> StrongPtr {
    unsafe { StrongPtr::new(NSString::alloc(nil).init_str(s)) }
}

enum NSImageScaling {
    NSImageScaleProportionallyUpOrDown = 3,
}

struct NSColor {
    r: f64,
    g: f64,
    b: f64,
    a: f64,
}

fn set_wallpaper(
    display_id: u64,
    wallpaper_path: &str,
    background_color: NSColor,
    scaling: NSImageScaling,
    allow_clipping: bool,
) {
    let screens = unsafe { NSScreen::screens(nil) };
    let screen_count = unsafe { NSArray::count(screens) };
    let mut screen_to_set: Option<*mut Object> = None;

    for i in 0..screen_count {
        let screen = unsafe { NSArray::objectAtIndex(screens, i) };
        let screen_description = unsafe { NSScreen::deviceDescription(screen) };
        let screen_id_obj =
            unsafe { NSDictionary::objectForKey_(screen_description, *nsstring("NSScreenNumber")) };
        let screen_id: u64 = unsafe { msg_send![screen_id_obj, unsignedIntValue] };
        if screen_id == display_id {
            screen_to_set = Some(screen);
            break;
        }
    }

    let screen_to_set = screen_to_set.unwrap();

    let workspace: id = unsafe { msg_send![class!(NSWorkspace), sharedWorkspace] };
    let wallpaper_url: id = unsafe { NSURL::fileURLWithPath_(nil, *nsstring(wallpaper_path)) };

    let rgba_color: id = unsafe {
        msg_send![class!(NSColor), colorWithSRGBRed:background_color.r green:background_color.g blue:background_color.b alpha:background_color.a]
    };

    let options_keys = vec![
        "NSWorkspaceDesktopImageScalingKey",
        "NSWorkspaceDesktopImageAllowClippingKey",
        "NSWorkspaceDesktopImageFillColorKey",
    ];
    let options_values: Vec<*mut Object> = vec![
        unsafe { msg_send![class!(NSNumber), numberWithInteger: scaling] },
        unsafe { msg_send![class!(NSNumber), numberWithInteger:(allow_clipping as u8)] },
        rgba_color,
    ];

    let mkstr = |s| unsafe { NSString::alloc(nil).init_str(s) };
    let keys_raw_vec = options_keys
        .clone()
        .into_iter()
        .map(&mkstr)
        .collect::<Vec<_>>();

    let keys_array = unsafe { NSArray::arrayWithObjects(nil, &keys_raw_vec) };
    let objs_array = unsafe { NSArray::arrayWithObjects(nil, &options_values) };

    let dict = unsafe { NSDictionary::dictionaryWithObjects_forKeys_(nil, objs_array, keys_array) };

    let _: id = unsafe {
        msg_send![workspace, setDesktopImageURL:wallpaper_url forScreen:screen_to_set options: dict error: nil]
    };
}

#[derive(Serialize, Deserialize, Debug)]
enum ImageScaling {
    #[serde(rename = "fit")]
    Fit,
    #[serde(rename = "fill")]
    Fill,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ImageSource {
    id: u64,
    url: String,
    estimated_size: String,
    update_interval: u64,
    dimensions: (u64, u64),
    #[serde(default)]
    is_thumbnail: bool,
    default_scaling: ImageScaling,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proj_dirs = ProjectDirs::from("com", "kydronepilot", "space-eye-rs").unwrap();
    let data_dir = proj_dirs.data_dir();
    let images_dir = data_dir.join("images");

    std::fs::create_dir_all(&images_dir).unwrap();

    let matches = Command::new("wallpaper")
        .arg(
            Arg::with_name("wallpaper")
                .help("Path to the wallpaper to set")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let wallpaper_path = matches.value_of("wallpaper").unwrap();
    set_wallpaper(
        1,
        wallpaper_path,
        NSColor {
            r: 1.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        },
        NSImageScaling::NSImageScaleProportionallyUpOrDown,
        true,
    );

    set_wallpaper(
        2,
        wallpaper_path,
        NSColor {
            r: 1.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        },
        NSImageScaling::NSImageScaleProportionallyUpOrDown,
        true,
    );

    Ok(())
}
