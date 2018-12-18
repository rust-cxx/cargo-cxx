extern crate cmake;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use std::env;
use std::fs;
use std::fs::File;
use std::fs::DirBuilder;
use std::fs::OpenOptions;
use std::path::Path;
use std::io::Write;
use std::io::Read;

pub enum LinkType
{
	Static,
	Dynamic,
	Executables,
}

pub struct Config
{
	curdir:String,
	pkg_name:String,
	profile:String,
	outpath:String,
	cmake_dir:std::path::PathBuf,
	link_type:LinkType,
	libs:Vec<String>,
	link_paths:Vec<String>,
	include_paths:Vec<String>,
}

impl Config
{
	pub fn new() -> Self
	{
		let outpath = env::var("OUT_DIR").ok().expect("Can't find OUT_DIR");
		let curdir = env::var("CARGO_MANIFEST_DIR").ok().expect("Can't find CARGO_MANIFEST_DIR");

		Self
		{
			pkg_name:env::var("CARGO_PKG_NAME").ok().expect("Can't find CARGO_PKG_NAME"),
			profile:env::var("PROFILE").ok().expect("Can't find PROFILE"),
			cmake_dir:Path::new(&outpath).join("../../../").join("cmake-links"),
			link_type:LinkType::Static,
			curdir:curdir.replace("\\","/"),
			outpath:outpath,
			libs:Vec::new(),
			link_paths:Vec::new(),
			include_paths:vec![curdir.replace("\\","/") + "/include"],
		}
	}

	pub fn link_type(&mut self, kind:LinkType)
	{
		self.link_type = kind;
	}

	pub fn set_pkg_name(&mut self, name:&str)
	{
		self.pkg_name = name.to_string();
	}

	pub fn add_include(&mut self, path:String)
	{
		self.include_paths.push(path);
	}

	pub fn add_link_path(&mut self, path:String)
	{
		self.link_paths.push(path);
	}

	pub fn add_lib(&mut self, path:String)
	{
		self.libs.push(path);
	}

	pub fn generator(&self) -> String
	{
		let mut cmake = "CMAKE_MINIMUM_REQUIRED(VERSION 3.0)".to_string();
		cmake += "\n\n";
		cmake += &format!("PROJECT({})", self.pkg_name);
		cmake += "\n\n";
		cmake += &format!("SET(LIB_NAME {})", self.pkg_name);
		cmake += "\n\n";
		cmake += &format!(r#"SET(PROJECT_ROOT_PATH "{}")"#, self.curdir);
		cmake += "\n\n";
		cmake += r#"FILE(GLOB_RECURSE HEADER_LIST "${PROJECT_ROOT_PATH}/include/*.h*")"#;
		cmake += "\n";
		cmake += r#"FILE(GLOB_RECURSE SOURCE_LIST "${PROJECT_ROOT_PATH}/source/*.c*")"#;
		cmake += "\n\n";
		cmake += r#"SOURCE_GROUP(${LIB_NAME} FILES ${HEADER_LIST})"#;
		cmake += "\n";
		cmake += r#"SOURCE_GROUP(${LIB_NAME} FILES ${SOURCE_LIST})"#;
		cmake += "\n\n";

		match self.link_type
		{
			LinkType::Static => 
			{
				cmake += r#"ADD_LIBRARY(${LIB_NAME} STATIC ${HEADER_LIST} ${SOURCE_LIST})"#;
				cmake += "\n\n";
			},
			LinkType::Dynamic => 
			{
				cmake += r#"ADD_LIBRARY(${LIB_NAME} SHARED ${HEADER_LIST} ${SOURCE_LIST})"#;
				cmake += "\n\n";
			},
			LinkType::Executables => 
			{
				cmake += r#"ADD_EXECUTABLE(${LIB_NAME} ${HEADER_LIST} ${SOURCE_LIST})"#;
				cmake += "\n\n";
			},
		}

		for path in self.link_paths.iter()
		{
			cmake += &format!(r#"TARGET_LINK_DIRECTORIES(${{LIB_NAME}} PRIVATE "{}")"#, path);
			cmake += "\n";
		}

		cmake += "\n";

		for inc in self.include_paths.iter()
		{
			cmake += &format!(r#"TARGET_INCLUDE_DIRECTORIES(${{LIB_NAME}} PRIVATE "{}")"#, inc);
			cmake += "\n";
		}

		cmake += "\n";

		for lib in self.libs.iter()
		{
			cmake += &format!(r#"TARGET_LINK_LIBRARIES(${{LIB_NAME}} PRIVATE "{}")"#, lib);
			cmake += "\n";
		}

		cmake += "\n";
		cmake += r#"INSTALL(DIRECTORY "${PROJECT_ROOT_PATH}/include/" DESTINATION include)"#;
		cmake += "\n";
		cmake += r#"INSTALL(TARGETS ${LIB_NAME} RUNTIME DESTINATION bin LIBRARY DESTINATION lib	ARCHIVE DESTINATION lib)"#;
		cmake
	}

	pub fn build(&mut self)
	{
		if Path::new("CMakeLists.txt").exists()
		{
			cmake::Config::new(".")
				.profile(&self.profile)
				.define("CMAKE_BUILD_TYPE", &self.profile)
				.build();
		}
		else
		{
			if !Path::new(&self.outpath).join("CMakeLists.txt").exists()
			{
				if Path::new(&self.cmake_dir).exists()
				{
					let entries = fs::read_dir(&self.cmake_dir).unwrap();
					for entry in entries
					{
						let path = entry.unwrap().path();
						let lib_name = path.file_stem().unwrap().to_str().unwrap().to_string();
						if lib_name != self.pkg_name
						{
							let mut file = File::open(path.as_path()).unwrap();
							let mut contents = String::new();
							file.read_to_string(&mut contents).unwrap();

							self.add_lib(lib_name);
							self.add_link_path(contents.replace("\\","/") + "/lib");
							self.add_include(contents.replace("\\","/") + "/include");
						}
					}
				}

				let mut file = File::create(&Path::new(&self.outpath).join("CMakeLists.txt")).unwrap();
				file.write(self.generator().as_bytes()).unwrap();
			}

			cmake::Config::new(&self.outpath)
				.profile(&self.profile)
				.define("CMAKE_BUILD_TYPE", &self.profile)
				.build();
		}

		DirBuilder::new()
			.recursive(true)
			.create(&self.cmake_dir).unwrap();

		let mut file = OpenOptions::new()
			.create(true)
			.write(true)
			.open(&self.cmake_dir.join(format!("{}", self.pkg_name))).unwrap();

		let mut contents = String::new();
		file.read_to_string(&mut contents);
		file.write((contents + &self.outpath).as_bytes()).unwrap();

		println!("cargo:rustc-link-search=native={}", self.outpath.clone() + "/lib");
		println!("cargo:rustc-link-lib=static={}", self.pkg_name);
	}

	pub fn build_exce(&mut self)
	{
		self.link_type = LinkType::Executables;
		self.build();
	}

	pub fn build_static_lib(&mut self)
	{
		self.link_type = LinkType::Static;
		self.build();
	}

	pub fn build_dynamic_lib(&mut self)
	{
		self.link_type = LinkType::Dynamic;
		self.build();
	}
}

pub fn new(name:&str)
{
	DirBuilder::new().recursive(true).create(&Path::new(&".").join(name)).unwrap();
	DirBuilder::new().recursive(true).create(&Path::new(&name.to_string()).join("include")).unwrap();
	DirBuilder::new().recursive(true).create(&Path::new(&name.to_string()).join("source")).unwrap();

	File::create(&Path::new(&".").join(name).join("Cargo.toml")).unwrap();

	let mut gitignore = File::create(&Path::new(&".").join(name).join(".gitignore")).unwrap();
	gitignore.write(b"/target\n**/*.rs.bk").unwrap();

	let mut build = File::create(&Path::new(&".").join(name).join("build.rs")).unwrap();
	build.write(b"fn main()\n{\n	cmake_cli::build();\n}").unwrap();
}