use rust_system_tools::wim::WimParser;
use std::fs::File;

/// 测试WIM解析器的架构解析功能
#[test]
fn test_parse_arch_from_xml() {
    let parser = WimParser::new_for_test(File::open("/dev/null").unwrap());

    // 测试各种架构值
    let test_cases = vec![
        (
            r#"<WINDOWS><ARCH>0</ARCH></WINDOWS>"#,
            Some("x86".to_string()),
        ),
        (
            r#"<WINDOWS><ARCH>9</ARCH></WINDOWS>"#,
            Some("x64".to_string()),
        ),
        (
            r#"<WINDOWS><ARCH>5</ARCH></WINDOWS>"#,
            Some("ARM".to_string()),
        ),
        (
            r#"<WINDOWS><ARCH>12</ARCH></WINDOWS>"#,
            Some("ARM64".to_string()),
        ),
        (r#"<WINDOWS><ARCH>999</ARCH></WINDOWS>"#, None),
        (r#"<WINDOWS></WINDOWS>"#, None),
    ];

    for (xml, expected) in test_cases {
        let result = parser.parse_arch_from_xml(xml);
        assert_eq!(result, expected, "测试XML: {}", xml);
    }
}

/// 测试单个镜像XML解析功能
#[test]
fn test_parse_single_image_xml_with_arch() {
    let parser = WimParser::new_for_test(File::open("/dev/null").unwrap());

    let xml = r#"<IMAGE INDEX="1">
        <DIRCOUNT>30978</DIRCOUNT>
        <FILECOUNT>136042</FILECOUNT>
        <TOTALBYTES>22577165103</TOTALBYTES>
        <WINDOWS>
            <ARCH>9</ARCH>
            <PRODUCTNAME>Microsoft® Windows® Operating System</PRODUCTNAME>
            <EDITIONID>Education</EDITIONID>
        </WINDOWS>
        <DISPLAYNAME>Windows 11 教育版</DISPLAYNAME>
        <DISPLAYDESCRIPTION>Windows 11 教育版</DISPLAYDESCRIPTION>
        <NAME>Windows 11 Education</NAME>
        <DESCRIPTION>Windows 11 Education</DESCRIPTION>
    </IMAGE>"#;

    let result = parser.parse_single_image_xml(xml);
    assert!(result.is_ok(), "解析应该成功");

    let image_info = result.unwrap();
    assert_eq!(image_info.index, 1);
    assert_eq!(image_info.name, "Windows 11 教育版");
    assert_eq!(image_info.architecture, Some("x64".to_string()));
    assert_eq!(image_info.version, Some("Windows 11".to_string()));
}

/// 测试不同架构值的解析
#[test]
fn test_different_arch_values() {
    let parser = WimParser::new_for_test(File::open("/dev/null").unwrap());

    // 测试x86架构
    let xml_x86 = r#"<IMAGE INDEX="1">
        <WINDOWS><ARCH>0</ARCH></WINDOWS>
        <DISPLAYNAME>Windows 10 Pro</DISPLAYNAME>
        <NAME>Windows 10 Pro</NAME>
    </IMAGE>"#;

    let result = parser.parse_single_image_xml(xml_x86).unwrap();
    assert_eq!(result.architecture, Some("x86".to_string()));

    // 测试ARM架构
    let xml_arm = r#"<IMAGE INDEX="2">
        <WINDOWS><ARCH>5</ARCH></WINDOWS>
        <DISPLAYNAME>Windows 10 Pro</DISPLAYNAME>
        <NAME>Windows 10 Pro</NAME>
    </IMAGE>"#;

    let result = parser.parse_single_image_xml(xml_arm).unwrap();
    assert_eq!(result.architecture, Some("ARM".to_string()));

    // 测试ARM64架构
    let xml_arm64 = r#"<IMAGE INDEX="3">
        <WINDOWS><ARCH>12</ARCH></WINDOWS>
        <DISPLAYNAME>Windows 11 Pro</DISPLAYNAME>
        <NAME>Windows 11 Pro</NAME>
    </IMAGE>"#;

    let result = parser.parse_single_image_xml(xml_arm64).unwrap();
    assert_eq!(result.architecture, Some("ARM64".to_string()));
}

/// 测试版本信息提取
#[test]
fn test_version_extraction() {
    let parser = WimParser::new_for_test(File::open("/dev/null").unwrap());

    let test_cases = vec![
        (
            "Windows 11 教育版",
            "Windows 11 教育版",
            Some("Windows 11".to_string()),
        ),
        (
            "Windows 10 Pro",
            "Windows 10 Pro",
            Some("Windows 10".to_string()),
        ),
        (
            "Windows Server 2022",
            "Windows Server 2022",
            Some("Windows Server 2022".to_string()),
        ),
        (
            "Windows Server 2019",
            "Windows Server 2019",
            Some("Windows Server 2019".to_string()),
        ),
        ("Unknown OS", "Unknown OS", None),
    ];

    for (name, description, expected_version) in test_cases {
        let xml = format!(
            r#"<IMAGE INDEX="1">
            <WINDOWS><ARCH>9</ARCH></WINDOWS>
            <DISPLAYNAME>{}</DISPLAYNAME>
            <NAME>{}</NAME>
        </IMAGE>"#,
            name, description
        );

        let result = parser.parse_single_image_xml(&xml).unwrap();
        assert_eq!(result.version, expected_version, "测试版本提取: {}", name);
    }
}

/// 测试架构优先级（XML中的ARCH标签优先于名称推断）
#[test]
fn test_architecture_priority() {
    let parser = WimParser::new_for_test(File::open("/dev/null").unwrap());

    // 名称中包含x86，但XML中ARCH标签为9（x64），应该优先使用XML中的值
    let xml = r#"<IMAGE INDEX="1">
        <WINDOWS><ARCH>9</ARCH></WINDOWS>
        <DISPLAYNAME>Windows 11 Pro x86</DISPLAYNAME>
        <NAME>Windows 11 Pro x86</NAME>
    </IMAGE>"#;

    let result = parser.parse_single_image_xml(xml).unwrap();
    assert_eq!(
        result.architecture,
        Some("x64".to_string()),
        "应该优先使用XML中的ARCH标签值，而不是名称中的架构信息"
    );
}

/// 测试回退机制（没有ARCH标签时从名称推断）
#[test]
fn test_fallback_architecture_detection() {
    let parser = WimParser::new_for_test(File::open("/dev/null").unwrap());

    // 没有ARCH标签，应该从名称推断
    let xml = r#"<IMAGE INDEX="1">
        <WINDOWS></WINDOWS>
        <DISPLAYNAME>Windows 11 Pro x64</DISPLAYNAME>
        <NAME>Windows 11 Pro x64</NAME>
    </IMAGE>"#;

    let result = parser.parse_single_image_xml(xml).unwrap();
    assert_eq!(
        result.architecture,
        Some("x64".to_string()),
        "没有ARCH标签时应该从名称推断架构"
    );
}
