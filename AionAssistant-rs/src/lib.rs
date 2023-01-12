#![feature(asm_const)]

mod lib_runner;
mod aion_assistant;
mod aion;

use std::error::Error;
use std::ffi::c_void;
use debug_print::{debug_eprintln, debug_println};
use hudhook::reexports::DLL_PROCESS_ATTACH;
use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::System::Console::{AllocConsole, FreeConsole, SetConsoleTitleA};
use windows::Win32::System::LibraryLoader::{DisableThreadLibraryCalls, FreeLibraryAndExitThread};

/*
    Useful sources:
    https://guidedhacking.com/threads/game-hacking-in-rust.15578/
    https://www.unknowncheats.me/forum/rust-language-/514796-game-memory-hacking-libraries-rust.html
    https://www.unknowncheats.me/forum/rust-language-/330583-pure-rust-injectable-dll.html
    https://github.com/pseuxide/toy-arms
    https://github.com/steele123/mem-rs

    Nav Mesh stuff:
    https://github.com/zzsort/monono2/blob/25eabe693a02ad999ec7574155e1448611c0a04a/ALGeoBuilder/NavMeshProcessor.cs#L16
    https://github.com/AionGermany/aion-germany/blob/master/AL-Game/src/com/aionemu/gameserver/world/WorldMapType.java
    https://github.com/zzsort/monono2/issues/1

    Other:
    https://stackoverflow.com/questions/40406225/how-to-add-parameters-to-a-running-process-or-exe-by-default
 */

fn dll_attach(hmodule_dll: &HINSTANCE) -> Result<(), Box<dyn Error>> {
	unsafe { lib_runner::main_loop(hmodule_dll); }

	// Return OK as main_loop has ended.
	Ok(())
}

// Wrapper which has to be unsafe to use and also depends on the system's own calling methodology
extern "system" fn dll_attach_wrapper(hmodule_dll: &HINSTANCE) -> u32 {

	// make sure that when attached, it doesn't fuck us over somehow (thru panicking)
	match std::panic::catch_unwind(|| dll_attach(hmodule_dll)) {
		Err(e) => {
			debug_eprintln!("`dll_attach` has panicked: {:#?}", e);
		}

		Ok(r) => match r {
			Ok(()) => {}
			Err(e) => {
				// something went wrong without panicking; time to tell us what it is and potentially diagnose
				debug_eprintln!("Error!");
				debug_eprintln!("Error!");
				debug_eprintln!("Error!");
				debug_eprintln!("Error!");
				debug_eprintln!("Error!");
				debug_eprintln!("`dll_attach` returned an Err: {:#?}", e);
				std::thread::sleep(std::time::Duration::from_secs(5));
			}
		},
	}

	debug_println!("Detaching!!");
	std::thread::sleep(std::time::Duration::from_secs(2));

	hudhook::lifecycle::eject();

	// We still manually free the console, library and then exit after eject()
	// because we never gave HUDHook's eject our HModule via global_state
	unsafe {
		// Detach the console, free the lib and exit this thread.
		FreeConsole();
		FreeLibraryAndExitThread(*hmodule_dll, 0);
	}
}

#[no_mangle] // call it "DllMain" in the compiled DLL
pub unsafe extern "stdcall" fn DllMain(hmodule_dll: HINSTANCE, fdw_reason: u32, _lpv_reserved: *mut c_void) -> i32 {
	if fdw_reason == DLL_PROCESS_ATTACH { // DLL_PROCESS_ATTACH == 1
		// Create debug terminal if in debug mode
		if cfg!(debug_assertions) {
			AllocConsole();

			// Set console title
			SetConsoleTitleA(windows::s!("AionAssistant-Rust dbg console"));
		}

		debug_println!("AionAssistant-Rust injected!");

		// TODO: What does this do exactly?
			DisableThreadLibraryCalls(hmodule_dll);

		// Create thread.  TODO: Does this create a leaking handle?
		std::thread::spawn(move || {
			dll_attach_wrapper(&hmodule_dll);
		});


		debug_println!("AionAssistant-Rust finished injecting!");
	}

	true as i32
}