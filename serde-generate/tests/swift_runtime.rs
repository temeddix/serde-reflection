// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde_generate::{
    swift, test_utils,
    test_utils::{Choice, Runtime, Test},
    CodeGeneratorConfig,
};
use std::{fs::File, io::Write, process::Command};

#[test]
fn test_swift_runtime_autotests() {
    let runtime_path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("../../../serde-generate/runtime/swift");

    let status = Command::new("swift")
        .current_dir(runtime_path.to_str().unwrap())
        .arg("test")
        .status()
        .unwrap();
    assert!(status.success());
}

// TODO: fix crash in runtime and enable test
#[test]
#[ignore]
fn test_swift_bcs_runtime_on_simple_data() {
    test_swift_runtime_on_simple_data(Runtime::Bcs);
}

#[test]
#[ignore]
fn test_swift_bincode_runtime_on_simple_data() {
    test_swift_runtime_on_simple_data(Runtime::Bincode);
}

fn test_swift_runtime_on_simple_data(runtime: Runtime) {
    // TODO: remove this and replace `my_path` by `dir.path()` below.
    let my_path = std::path::Path::new("/Users/mathieubaudet/git/serde-reflection/test");
    std::fs::remove_dir_all(my_path).unwrap_or(());
    std::fs::create_dir_all(my_path).unwrap();
    // let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(my_path.join("Sources/Testing")).unwrap();
    std::fs::create_dir_all(my_path.join("Sources/main")).unwrap();
    let serde_package_path = std::env::current_dir()
        .unwrap()
        .join("../serde-generate/runtime/swift");
    let mut file = File::create(my_path.join("Package.swift")).unwrap();
    write!(
        file,
        r#"// swift-tools-version:5.3

import PackageDescription

let package = Package(
    name: "Testing",
    dependencies: [
        .package(name: "Serde", path: "{}"),
    ],
    targets: [
        .target(
            name: "Testing",
            dependencies: ["Serde"]),
        .target(
            name: "main",
            dependencies: ["Serde", "Testing"]
        ),
    ]
)
"#,
        serde_package_path.to_str().unwrap()
    )
    .unwrap();

    let codegen_path = my_path.join("Sources/Testing/Testing.swift");
    let mut codegen = File::create(&codegen_path).unwrap();
    let config =
        CodeGeneratorConfig::new("Testing".to_string()).with_encodings(vec![runtime.into()]);
    let registry = test_utils::get_simple_registry().unwrap();
    let generator = swift::CodeGenerator::new(&config);
    generator.output(&mut codegen, &registry).unwrap();

    let reference = runtime.serialize(&Test {
        a: vec![4, 6],
        b: (-3, 5),
        c: Choice::C { x: 7 },
    });

    let main_path = my_path.join("Sources/main/main.swift");
    let mut main = File::create(main_path).unwrap();
    writeln!(
        main,
        r#"
import Serde
import Testing

let input : [UInt8] = [{0}]
let value = try Test.{1}Deserialize(input: input)

let value2 = Test.init(
    a: [4, 6],
    b: Tuple2.init(-3, 5),
    c: Choice.C(x: 7)
)
if (value != value2) {{ assertionFailure("value != value2") }}

let output = try value2.{1}Serialize()
if (input != output)  {{ assertionFailure("input != output") }}

do {{
    let input2 : [UInt8] = [0, 1]
    let _ = try Test.{1}Deserialize(input: input2)
    assertionFailure("Was expecting an error")
}}
catch {{}}
"#,
        reference
            .iter()
            .map(|x| format!("{}", x))
            .collect::<Vec<_>>()
            .join(", "),
        runtime.name(),
    )
    .unwrap();

    let status = Command::new("swift")
        .current_dir(my_path)
        .arg("run")
        .status()
        .unwrap();
    assert!(status.success());
}