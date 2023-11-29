// #include "MemoryEdits.h"

// #include <Windows.h>
// #include <tlhelp32.h>


// ullong GetModuleBase(const int ProcessID, const char * ModuleName);

//add the pointer offsets to the address
// ullong FindPointerAddr(HANDLE pHandle, const ullong baseaddr, const size_t length, const int * offsets);

use std::ffi::{c_char, c_void, CStr};
use std::mem::size_of;
use std::ptr::null_mut;
use windows::Win32::Foundation::{CloseHandle, HANDLE, HMODULE, INVALID_HANDLE_VALUE};
use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use windows::Win32::System::Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Module32Next, MODULEENTRY32, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPMODULE, TH32CS_SNAPPROCESS};


// TODO: looks like shit, rewrite
//get the process id from the name
pub unsafe fn GetProcessIdFromName(ProcName: &str) -> u32 {
	let mut entry = PROCESSENTRY32 {
		dwSize: 0,
		cntUsage: 0,
		th32ProcessID: 0,
		th32DefaultHeapID: 0,
		th32ModuleID: 0,
		cntThreads: 0,
		th32ParentProcessID: 0,
		pcPriClassBase: 0,
		dwFlags: 0,
		szExeFile: [0; 260],
	};

	entry.dwSize = size_of::<PROCESSENTRY32>() as u32;
	let mut processid: u32 = u32::MAX;
	//get all running processes
	let hSnapshot: HANDLE = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).unwrap(); // TODO

	//search through all running programs until we find the one that matches
	if Process32First(hSnapshot, &mut entry as *mut PROCESSENTRY32).is_ok() {
		loop {
			// let cstr: &CStr = CStr::from_ptr(&entry.szExeFile as *const c_char);
			// let cstr: String = cstr.into_c_string().into_string().unwrap(); // TODO: handle error
			// let cstr: String = cstr.indi
			let pn = ProcName.to_lowercase();
			let mut processname: [u8;20] = [0;20];//just go with 20
			processname.copy_from_slice(&entry.szExeFile[..20]);
			let processname = CStr::from_ptr(&processname as *const _ as *const c_char)
				.to_str().unwrap().to_lowercase(); // TODO: handle error
			// size_t charsConverted = 0;
			// wcstombs_s(&charsConverted, processname, 20, entry.szExeFile, 19);
			//compare the process name and desired process name
			if pn == processname {
				processid = entry.th32ProcessID;
				break;
			}
			Process32Next(hSnapshot, &mut entry as *mut PROCESSENTRY32).unwrap(); // TODO: handle error
		}
	}

	if hSnapshot != INVALID_HANDLE_VALUE {
		CloseHandle(hSnapshot).unwrap(); // TODO: handle error
	}

	return processid;
}

// find base address of process
// expected u64 on the caller site
pub unsafe fn GetModuleBase(ProcessID: u32, ModuleName: &str) -> u64 {
	//go through the programs loaded module's and find the primary one (same name as program process)
	let mut hSnap: HANDLE = HANDLE(0);
	let mut Mod32: MODULEENTRY32 = MODULEENTRY32 {
		dwSize: 0,
		th32ModuleID: 0,
		th32ProcessID: 0,
		GlblcntUsage: 0,
		ProccntUsage: 0,
		modBaseAddr: null_mut(),
		modBaseSize: 0,
		hModule: HMODULE(0),
		szModule: [0; 256],
		szExePath: [0; 260],
	};

	hSnap = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, ProcessID).unwrap(); // TODO: handle error

	if hSnap == INVALID_HANDLE_VALUE {
		return Mod32.modBaseAddr as u64;
	}

	Mod32.dwSize = size_of::<MODULEENTRY32>() as u32;
	while Module32Next(hSnap, &mut Mod32).is_ok() {
		// TODO: looks like shit, rewrite
		let mn = ModuleName.to_lowercase();
		let mut processname: [u8;20] = [0;20];//just go with 20
		processname.copy_from_slice(&Mod32.szModule[..20]);
		let szModule = CStr::from_ptr(&processname as *const _ as *const c_char)
			.to_str().unwrap().to_lowercase(); // TODO: handle error

		//if the module name matches the process name
		if mn == szModule {
			CloseHandle(hSnap).unwrap(); // TODO: handle error
			return Mod32.modBaseAddr as u64;
		}
	}

	CloseHandle(hSnap).unwrap(); // TODO: handle error
	return 0;
}

pub unsafe fn FindPointerAddr(pHandle: HANDLE, baseaddr: u64, length: usize, offsets: &[u64]) -> u64 {
	let mut address: u64 = baseaddr;

	for i in 0..length {
		ReadProcessMemory(pHandle, address as *const c_void, &mut address as *mut _ as *mut c_void, 4, None).unwrap(); // TODO: handle error
		address += offsets[i];
	}

	return address;
}