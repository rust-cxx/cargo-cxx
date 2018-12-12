extern crate cmake;

use std::env;
use std::fs::File;
use std::fs::DirBuilder;
use std::path::Path;
use std::io::Write;

use argparse::Store;
use argparse::ArgumentParser;

fn generator_cmake(target:&str, curdir:&str) -> String
{
	let mut cmake = "CMAKE_MINIMUM_REQUIRED(VERSION 3.0)".to_string();
	cmake += "\n\n";
	cmake += &format!("PROJECT({})", target);
	cmake += "\n\n";
	cmake += &format!("SET(LIB_NAME {})", target);
	cmake += "\n\n";
	cmake += &format!(r#"SET(PROJECT_ROOT_PATH "{}")"#, curdir.replace("\\","/"));
	cmake += "\n\n";
	cmake += r#"FILE(GLOB_RECURSE HEADER_LIST "${PROJECT_ROOT_PATH}/include/*.h*")"#;
	cmake += "\n\n";
	cmake += r#"FILE(GLOB_RECURSE SOURCE_LIST "${PROJECT_ROOT_PATH}/source/*.c*")"#;
	cmake += "\n\n";
	cmake += r#"SOURCE_GROUP(${LIB_NAME} FILES ${HEADER_LIST})"#;
	cmake += "\n\n";
	cmake += r#"SOURCE_GROUP(${LIB_NAME} FILES ${SOURCE_LIST})"#;
	cmake += "\n\n";
	cmake += r#"ADD_LIBRARY(${LIB_NAME} STATIC ${HEADER_LIST} ${SOURCE_LIST})"#;
	cmake += "\n\n";
	cmake += r#"TARGET_LINK_LIBRARIES(${LIB_NAME} PRIVATE imgui)"#;
	cmake += "\n\n";
	cmake += r#"TARGET_INCLUDE_DIRECTORIES(${LIB_NAME} PRIVATE "${PROJECT_ROOT_PATH}/include")"#;
	cmake += "\n\n";
	cmake += r#"INSTALL(DIRECTORY "${PROJECT_ROOT_PATH}/include/" DESTINATION include)"#;
	cmake += "\n\n";
	cmake += r#"INSTALL(TARGETS ${LIB_NAME} RUNTIME DESTINATION bin LIBRARY DESTINATION lib	ARCHIVE DESTINATION lib)"#;
	cmake += "\n\n";
	cmake
}

pub fn build()
{
	let outpath = env::var("OUT_DIR").ok().expect("Can't find OUT_DIR");
	let pkg_name = env::var("CARGO_PKG_NAME").ok().expect("Can't find CARGO_PKG_NAME");
	let profile = env::var("PROFILE").ok().expect("Can't find PROFILE");

	if Path::new("CMakeLists.txt").exists()
	{
		cmake::Config::new(".")
			.profile(&profile)
			.define("CMAKE_BUILD_TYPE", &profile)
			.build();
	}
	else
	{
		if !Path::new(&outpath).join("CMakeLists.txt").exists()
		{
			let curdir = env::var("CARGO_MANIFEST_DIR").ok().expect("Can't find CARGO_MANIFEST_DIR");

			let mut file = File::create(&Path::new(&outpath).join("CMakeLists.txt")).unwrap();
			file.write(generator_cmake(&pkg_name, &curdir).as_bytes()).unwrap();
		}

		cmake::Config::new(&outpath)
			.profile(&profile)
			.define("CMAKE_BUILD_TYPE", &profile)
			.build();
	}

	println!("cargo:rustc-link-search=native={}", outpath+"/lib");
	println!("cargo:rustc-link-lib=static={}", pkg_name);
}

pub fn new(name:&str)
{
	DirBuilder::new().recursive(true).create(&Path::new(&".").join(name)).unwrap();
	DirBuilder::new().recursive(true).create(&Path::new(&name.to_string()).join("include")).unwrap();
	DirBuilder::new().recursive(true).create(&Path::new(&name.to_string()).join("source")).unwrap();

	File::create(&Path::new(&".").join(name).join("Cargo.toml")).unwrap();

	let mut gitignore = File::create(&Path::new(&".").join(name).join(".gitignore")).unwrap();
	gitignore.write(b"/target\n**/*.rs.bk").unwrap();
}