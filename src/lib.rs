extern crate cmake;

use std::env;
use std::fs;
use std::fs::File;
use std::fs::DirBuilder;
use std::fs::OpenOptions;
use std::path::Path;
use std::io::Write;
use std::io::Read;

#[derive(Debug, PartialEq)]
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
	cmake_dir:String,
	link_type:LinkType,
	libs:Vec<String>,
	defines:Vec<(String, String)>,
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
			cmake_dir:outpath.clone() + "../../../../cmake-links",
			link_type:LinkType::Static,
			curdir:curdir.replace("\\","/"),
			outpath:outpath,
			libs:Vec::new(),
			defines:Vec::new(),
			link_paths:Vec::new(),
			include_paths:vec![curdir.replace("\\","/") + "/include"],
		}
	}

	pub fn project(&mut self, name:&str) -> &mut Self
	{
		self.pkg_name = name.to_string();
		self
	}

	pub fn profile(&mut self, profile:&str) -> &mut Self
	{
		self.profile = profile.to_string();
		self
	}

	pub fn link_type(&mut self, kind:LinkType) -> &mut Self
	{
		self.link_type = kind;
		self
	}

	pub fn include(&mut self, path:&str) -> &mut Self
	{
		self.include_paths.push(path.to_string());
		self
	}

	pub fn link_path(&mut self, path:&str) -> &mut Self
	{
		self.link_paths.push(path.to_string());
		self
	}

	pub fn link(&mut self, path:&str) -> &mut Self
	{
		self.libs.push(path.to_string());
		self
	}

	pub fn define(&mut self, name:&str, value:&str) -> &mut Self
	{
		self.defines.push((name.to_string(), value.to_string()));
		self
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
		cmake += "SET(CMAKE_CXX_STANDARD 17)\n";
		cmake += "SET(CMAKE_CXX_STANDARD_REQUIRED ON)\n";
		cmake += "SET(CMAKE_CXX_EXTENSIONS OFF)\n";

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

		for def in self.defines.iter()
		{
			cmake += &format!(r#"TARGET_COMPILE_DEFINITIONS(${{LIB_NAME}} PRIVATE "{}={}")"#, def.0, def.1);
			cmake += "\n";
		}

		cmake += "\n";

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
		{
			DirBuilder::new().recursive(true).create(&Path::new(&self.cmake_dir)).unwrap();

			let mut file = OpenOptions::new()
				.create(true)
				.write(true)
				.open(&Path::new(&self.cmake_dir).join(format!("{}", self.pkg_name))).unwrap();

			file.write(&self.outpath.as_bytes()).unwrap();
		}

		if Path::new("CMakeLists.txt").exists()
		{
			let mut make = cmake::Config::new(".");
			make.profile(&self.profile);
			make.define("CMAKE_BUILD_TYPE", &self.profile);

			for def in self.defines.iter()
			{
				make.define(&def.0, &def.1);
			}

			make.build();
		}
		else
		{
			if !Path::new(&self.outpath).join("CMakeLists.txt").exists()
			{
				if Path::new(&self.cmake_dir).exists()
				{
					let entries = fs::read_dir(&Path::new(&self.cmake_dir)).unwrap();
					for entry in entries
					{
						let path = entry.unwrap().path();
						let lib_name = path.file_stem().unwrap().to_str().unwrap().to_string();
						if lib_name != self.pkg_name
						{
							let mut file = File::open(path.as_path()).unwrap();
							let mut contents = String::new();
							file.read_to_string(&mut contents).unwrap();

							self.link(&lib_name);
							self.link_path(&(contents.replace("\\","/") + "/lib"));
							self.include(&(contents.replace("\\","/") + "/include"));
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

		if self.link_type != LinkType::Executables
		{
			println!("cargo:rustc-link-search=native={}", self.outpath.clone() + "/lib");
			println!("cargo:rustc-link-lib=static={}", self.pkg_name);
		}
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
	build.write(b"fn main()\n{\n	cxx::Config::new().build();\n}").unwrap();
}