use anyhow::{Context, Result};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use tracing::{debug, info};

/// WIM 文件头结构体 (WIMHEADER_V1_PACKED)
/// 总大小：204 字节
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WimHeader {
    /// 文件签名 "MSWIM\x00\x00\x00"
    pub signature: [u8; 8],
    /// 文件头大小
    pub header_size: u32,
    /// 格式版本
    pub format_version: u32,
    /// 文件标志
    pub file_flags: u32,
    /// 压缩文件大小
    pub compressed_size: u32,
    /// 唯一标识符 (GUID)
    pub guid: [u8; 16],
    /// 段号
    pub segment_number: u16,
    /// 段总数
    pub total_segments: u16,
    /// 镜像数量
    pub image_count: u32,
    /// 偏移表文件资源
    pub offset_table_resource: FileResourceEntry,
    /// XML 数据文件资源
    pub xml_data_resource: FileResourceEntry,
    /// 引导元数据文件资源
    pub boot_metadata_resource: FileResourceEntry,
    /// 可引导镜像索引
    pub bootable_image_index: u32,
    /// 完整性数据文件资源
    pub integrity_resource: FileResourceEntry,
}

/// 文件资源条目结构体 (_RESHDR_DISK_SHORT)
/// 总大小：24 字节
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FileResourceEntry {
    /// 资源大小 (7 字节)
    pub size: u64,
    /// 资源标志 (1 字节)
    pub flags: u8,
    /// 资源偏移 (8 字节)
    pub offset: u64,
    /// 原始大小 (8 字节)
    pub original_size: u64,
}

/// 文件资源条目标志
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ResourceFlags;

#[allow(dead_code)]
impl ResourceFlags {
    pub const FREE: u8 = 0x01; // 条目空闲
    pub const METADATA: u8 = 0x02; // 包含元数据
    pub const COMPRESSED: u8 = 0x04; // 已压缩
    pub const SPANNED: u8 = 0x08; // 跨段
}

/// 文件标志
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FileFlags;

#[allow(dead_code)]
impl FileFlags {
    pub const COMPRESSION: u32 = 0x00000002; // 资源已压缩
    pub const READONLY: u32 = 0x00000004; // 只读
    pub const SPANNED: u32 = 0x00000008; // 跨段
    pub const RESOURCE_ONLY: u32 = 0x00000010; // 仅包含文件资源
    pub const METADATA_ONLY: u32 = 0x00000020; // 仅包含元数据
    pub const COMPRESS_XPRESS: u32 = 0x00020000; // XPRESS 压缩
    pub const COMPRESS_LZX: u32 = 0x00040000; // LZX 压缩
}

/// 镜像信息结构体
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ImageInfo {
    /// 镜像索引
    pub index: u32,
    /// 镜像名称
    pub name: String,
    /// 镜像描述
    pub description: String,
    /// 目录数量
    pub dir_count: u32,
    /// 文件数量
    pub file_count: u32,
    /// 总字节数
    pub total_bytes: u64,
    /// 创建时间
    pub creation_time: Option<u64>,
    /// 最后修改时间
    pub last_modification_time: Option<u64>,
    /// 版本信息
    pub version: Option<String>,
    /// 架构信息
    pub architecture: Option<String>,
}

/// WIM 文件解析器
pub struct WimParser {
    file: File,
    header: Option<WimHeader>,
    images: Vec<ImageInfo>,
}

impl WimParser {
    /// 创建新的 WIM 解析器
    pub fn new<P: AsRef<Path>>(wim_path: P) -> Result<Self> {
        let file = File::open(wim_path.as_ref())
            .with_context(|| format!("无法打开 WIM 文件: {}", wim_path.as_ref().display()))?;

        debug!("创建 WIM 解析器: {}", wim_path.as_ref().display());

        Ok(Self {
            file,
            header: None,
            images: Vec::new(),
        })
    }

    /// 创建用于测试的 WIM 解析器（不需要实际文件）
    #[doc(hidden)]
    #[allow(dead_code)]
    pub fn new_for_test(file: File) -> Self {
        Self {
            file,
            header: None,
            images: Vec::new(),
        }
    }

    /// 读取并解析 WIM 文件头
    pub fn read_header(&mut self) -> Result<&WimHeader> {
        if self.header.is_some() {
            return Ok(self.header.as_ref().unwrap());
        }

        debug!("开始读取 WIM 文件头");

        // 跳转到文件开始
        self.file.seek(SeekFrom::Start(0))?;

        // 读取 204 字节的文件头
        let mut header_buffer = vec![0u8; 204];
        self.file
            .read_exact(&mut header_buffer)
            .context("读取 WIM 文件头失败")?;

        let header = self.parse_header_buffer(&header_buffer)?;

        // 验证签名
        if &header.signature != b"MSWIM\x00\x00\x00" {
            return Err(anyhow::anyhow!("无效的 WIM 文件签名"));
        }

        info!(
            "成功读取 WIM 文件头 - 版本: {}, 镜像数: {}",
            header.format_version, header.image_count
        );

        self.header = Some(header);
        Ok(self.header.as_ref().unwrap())
    }

    /// 解析文件头缓冲区
    fn parse_header_buffer(&self, buffer: &[u8]) -> Result<WimHeader> {
        use std::convert::TryInto;

        // 辅助函数：从缓冲区读取 little-endian 数值
        let read_u32_le = |offset: usize| -> u32 {
            u32::from_le_bytes(buffer[offset..offset + 4].try_into().unwrap())
        };

        let read_u16_le = |offset: usize| -> u16 {
            u16::from_le_bytes(buffer[offset..offset + 2].try_into().unwrap())
        };

        let read_u64_le = |offset: usize| -> u64 {
            u64::from_le_bytes(buffer[offset..offset + 8].try_into().unwrap())
        };

        // 解析文件资源条目
        let parse_resource_entry = |offset: usize| -> FileResourceEntry {
            // 读取 7 字节的大小 + 1 字节标志
            let size_bytes = &buffer[offset..offset + 7];
            let mut size_array = [0u8; 8];
            size_array[..7].copy_from_slice(size_bytes);
            let size = u64::from_le_bytes(size_array);

            let flags = buffer[offset + 7];
            let offset_val = read_u64_le(offset + 8);
            let original_size = read_u64_le(offset + 16);

            FileResourceEntry {
                size,
                flags,
                offset: offset_val,
                original_size,
            }
        };

        // 解析文件头各个字段
        let mut signature = [0u8; 8];
        signature.copy_from_slice(&buffer[0..8]);

        let header = WimHeader {
            signature,
            header_size: read_u32_le(8),
            format_version: read_u32_le(12),
            file_flags: read_u32_le(16),
            compressed_size: read_u32_le(20),
            guid: buffer[24..40].try_into().unwrap(),
            segment_number: read_u16_le(40),
            total_segments: read_u16_le(42),
            image_count: read_u32_le(44),
            offset_table_resource: parse_resource_entry(48),
            xml_data_resource: parse_resource_entry(72),
            boot_metadata_resource: parse_resource_entry(96),
            bootable_image_index: read_u32_le(120),
            integrity_resource: parse_resource_entry(124),
        };

        debug!(
            "解析 WIM 头部完成 - 镜像数: {}, 文件标志: 0x{:08X}",
            header.image_count, header.file_flags
        );

        Ok(header)
    }

    /// 读取并解析 XML 数据
    pub fn read_xml_data(&mut self) -> Result<()> {
        // 确保文件头已读取
        if self.header.is_none() {
            self.read_header()?;
        }

        let header = self.header.as_ref().unwrap();

        // 检查 XML 数据资源是否存在
        if header.xml_data_resource.size == 0 {
            return Err(anyhow::anyhow!("WIM 文件中没有 XML 数据资源"));
        }

        debug!(
            "开始读取 XML 数据，偏移: {}, 大小: {}",
            header.xml_data_resource.offset, header.xml_data_resource.size
        );

        // 跳转到 XML 数据位置
        self.file
            .seek(SeekFrom::Start(header.xml_data_resource.offset))?;

        // 读取 XML 数据
        let mut xml_buffer = vec![0u8; header.xml_data_resource.size as usize];
        self.file
            .read_exact(&mut xml_buffer)
            .context("读取 XML 数据失败")?;

        // 解析 XML 数据
        self.parse_xml_data(&xml_buffer)?;

        info!("成功解析 {} 个镜像的信息", self.images.len());
        Ok(())
    }

    /// 解析 XML 数据
    fn parse_xml_data(&mut self, xml_buffer: &[u8]) -> Result<()> {
        // XML 数据以 UTF-16 LE BOM 开始
        if xml_buffer.len() < 2 {
            return Err(anyhow::anyhow!("XML 数据太短"));
        }

        // 检查 BOM (0xFEFF)
        if xml_buffer[0] != 0xFF || xml_buffer[1] != 0xFE {
            return Err(anyhow::anyhow!("无效的 XML 数据 BOM"));
        }

        // 将 UTF-16 LE 转换为 UTF-8
        let xml_utf16_data = &xml_buffer[2..]; // 跳过 BOM

        // 确保数据长度为偶数（UTF-16 每个字符 2 字节）
        if xml_utf16_data.len() % 2 != 0 {
            return Err(anyhow::anyhow!("XML UTF-16 数据长度不是偶数"));
        }

        // 转换为 u16 数组
        let mut utf16_chars = Vec::new();
        for chunk in xml_utf16_data.chunks_exact(2) {
            let char_val = u16::from_le_bytes([chunk[0], chunk[1]]);
            utf16_chars.push(char_val);
        }

        // 转换为 UTF-8 字符串
        let xml_string = String::from_utf16(&utf16_chars).context("无法将 XML 数据转换为 UTF-8")?;

        debug!("XML 数据长度: {} 字符", xml_string.len());

        // 解析 XML 镜像信息
        self.parse_xml_images(&xml_string)?;

        Ok(())
    }

    /// 解析 XML 中的镜像信息
    fn parse_xml_images(&mut self, xml_content: &str) -> Result<()> {
        // 简单的 XML 解析（基于字符串匹配）
        // 在实际生产环境中，建议使用专门的 XML 解析库

        self.images.clear();

        // 查找所有 <IMAGE> 标签
        let mut start_pos = 0;
        while let Some(image_start) = xml_content[start_pos..].find("<IMAGE") {
            let absolute_start = start_pos + image_start;

            // 查找对应的 </IMAGE> 标签
            if let Some(image_end) = xml_content[absolute_start..].find("</IMAGE>") {
                let absolute_end = absolute_start + image_end + 8; // 包含 </IMAGE>
                let image_xml = &xml_content[absolute_start..absolute_end];

                // 解析单个镜像信息
                if let Ok(image_info) = self.parse_single_image_xml(image_xml) {
                    self.images.push(image_info);
                }

                start_pos = absolute_end;
            } else {
                break;
            }
        }

        Ok(())
    }

    /// 解析单个镜像的 XML 信息
    pub fn parse_single_image_xml(&self, image_xml: &str) -> Result<ImageInfo> {
        // 辅助函数：从 XML 中提取标签值
        let extract_tag_value = |xml: &str, tag: &str| -> Option<String> {
            let start_tag = format!("<{tag}>");
            let end_tag = format!("</{tag}>");

            if let Some(start) = xml.find(&start_tag) {
                if let Some(end) = xml.find(&end_tag) {
                    let value_start = start + start_tag.len();
                    if value_start < end {
                        return Some(xml[value_start..end].trim().to_string());
                    }
                }
            }
            None
        };

        // 提取 INDEX 属性
        let index = if let Some(index_start) = image_xml.find("INDEX=\"") {
            let index_value_start = index_start + 7; // "INDEX=\"".len()
            if let Some(index_end) = image_xml[index_value_start..].find("\"") {
                let index_str = &image_xml[index_value_start..index_value_start + index_end];
                index_str.parse().unwrap_or(0)
            } else {
                0
            }
        } else {
            0
        };

        // 提取各种信息
        let name =
            extract_tag_value(image_xml, "DISPLAYNAME").unwrap_or_else(|| format!("Image {index}"));
        let description = extract_tag_value(image_xml, "DISPLAYDESCRIPTION")
            .unwrap_or_else(|| "Unknown".to_string());
        let dir_count = extract_tag_value(image_xml, "DIRCOUNT")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let file_count = extract_tag_value(image_xml, "FILECOUNT")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let total_bytes = extract_tag_value(image_xml, "TOTALBYTES")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        // 尝试从XML中的ARCH标签解析架构信息
        let arch_from_xml = self.parse_arch_from_xml(image_xml);

        // 从名称中提取版本信息，架构信息优先使用XML中的ARCH标签
        let (version, arch_from_name) = self.extract_version_and_arch(&name, &description);
        let architecture = arch_from_xml.or(arch_from_name);

        let image_info = ImageInfo {
            index,
            name,
            description,
            dir_count,
            file_count,
            total_bytes,
            creation_time: None,          // 可以进一步解析 CREATIONTIME
            last_modification_time: None, // 可以进一步解析 LASTMODIFICATIONTIME
            version,
            architecture,
        };

        debug!(
            "解析镜像信息: {} - {} - {} - {:#?}",
            image_info.index, image_info.name, image_info.description, image_info.architecture
        );

        Ok(image_info)
    }

    /// 从镜像名称和描述中提取版本和架构信息
    fn extract_version_and_arch(
        &self,
        name: &str,
        description: &str,
    ) -> (Option<String>, Option<String>) {
        let combined_text = format!("{name} {description}").to_lowercase();

        // 提取版本信息
        let version = if combined_text.contains("windows 11") {
            Some("Windows 11".to_string())
        } else if combined_text.contains("windows 10") {
            Some("Windows 10".to_string())
        } else if combined_text.contains("windows server 2022") {
            Some("Windows Server 2022".to_string())
        } else if combined_text.contains("windows server 2019") {
            Some("Windows Server 2019".to_string())
        } else if combined_text.contains("windows server") {
            Some("Windows Server".to_string())
        } else if combined_text.contains("windows") {
            Some("Windows".to_string())
        } else {
            None
        };

        // 提取架构信息
        let architecture = if combined_text.contains("x64") || combined_text.contains("amd64") {
            Some("x64".to_string())
        } else if combined_text.contains("x86") {
            Some("x86".to_string())
        } else if combined_text.contains("arm64") {
            Some("ARM64".to_string())
        } else {
            None
        };

        (version, architecture)
    }

    /// 从XML中的ARCH标签解析架构信息
    pub fn parse_arch_from_xml(&self, image_xml: &str) -> Option<String> {
        // 辅助函数：从 XML 中提取标签值
        let extract_tag_value = |xml: &str, tag: &str| -> Option<String> {
            let start_tag = format!("<{tag}>");
            let end_tag = format!("</{tag}>");

            if let Some(start) = xml.find(&start_tag) {
                if let Some(end) = xml.find(&end_tag) {
                    let value_start = start + start_tag.len();
                    if value_start < end {
                        return Some(xml[value_start..end].trim().to_string());
                    }
                }
            }
            None
        };

        // 提取ARCH标签值
        if let Some(arch_value) = extract_tag_value(image_xml, "ARCH") {
            match arch_value.as_str() {
                "0" => Some("x86".to_string()),
                "9" => Some("x64".to_string()),
                "5" => Some("ARM".to_string()),
                "12" => Some("ARM64".to_string()),
                _ => {
                    debug!("未知的架构值: {}", arch_value);
                    None
                }
            }
        } else {
            None
        }
    }

    /// 获取所有镜像信息
    pub fn get_images(&self) -> &[ImageInfo] {
        &self.images
    }

    /// 获取指定索引的镜像信息
    #[allow(dead_code)]
    pub fn get_image(&self, index: u32) -> Option<&ImageInfo> {
        self.images.iter().find(|img| img.index == index)
    }

    /// 获取文件头信息
    #[allow(dead_code)]
    pub fn get_header(&self) -> Option<&WimHeader> {
        self.header.as_ref()
    }

    /// 检查是否包含多个镜像
    #[allow(dead_code)]
    pub fn has_multiple_images(&self) -> bool {
        self.header
            .as_ref()
            .map(|h| h.image_count > 1)
            .unwrap_or(false)
    }

    /// 获取镜像数量
    #[allow(dead_code)]
    pub fn get_image_count(&self) -> u32 {
        self.header.as_ref().map(|h| h.image_count).unwrap_or(0)
    }

    /// 检查是否为压缩文件
    #[allow(dead_code)]
    pub fn is_compressed(&self) -> bool {
        self.header
            .as_ref()
            .map(|h| h.file_flags & FileFlags::COMPRESSION != 0)
            .unwrap_or(false)
    }

    /// 获取压缩类型
    #[allow(dead_code)]
    pub fn get_compression_type(&self) -> Option<&'static str> {
        if let Some(header) = &self.header {
            if header.file_flags & FileFlags::COMPRESS_XPRESS != 0 {
                Some("XPRESS")
            } else if header.file_flags & FileFlags::COMPRESS_LZX != 0 {
                Some("LZX")
            } else if header.file_flags & FileFlags::COMPRESSION != 0 {
                Some("Unknown")
            } else {
                None
            }
        } else {
            None
        }
    }

    /// 完整解析 WIM 文件（头部 + XML 数据）
    pub fn parse_full(&mut self) -> Result<()> {
        self.read_header()?;
        self.read_xml_data()?;
        Ok(())
    }
}

impl std::fmt::Display for ImageInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "镜像 {} - {}", self.index, self.name)?;
        if let Some(ref version) = self.version {
            write!(f, " [{version}]")?;
        }
        if let Some(ref arch) = self.architecture {
            write!(f, " [{arch}]")?;
        }
        write!(f, " | 描述: {}", self.description)?;
        write!(
            f,
            " | 文件数: {}, 目录数: {}",
            self.file_count, self.dir_count
        )?;
        write!(f, " | 总大小: {} MB", self.total_bytes / (1024 * 1024))?;
        Ok(())
    }
}

impl std::fmt::Display for WimHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "WIM Header:")?;
        writeln!(f, "  Format Version: {}", self.format_version)?;
        writeln!(f, "  File Flags: 0x{:08X}", self.file_flags)?;
        writeln!(f, "  Image Count: {}", self.image_count)?;
        writeln!(
            f,
            "  Segment: {}/{}",
            self.segment_number, self.total_segments
        )?;
        writeln!(f, "  Bootable Image Index: {}", self.bootable_image_index)?;
        Ok(())
    }
}

#[allow(dead_code)]
impl WimParser {
    /// 获取所有镜像的版本摘要
    #[allow(dead_code)]
    pub fn get_version_summary(&self) -> Vec<String> {
        let mut summaries = Vec::new();

        for image in &self.images {
            let mut summary = format!("镜像 {}: {}", image.index, image.name);

            if let Some(ref version) = image.version {
                summary.push_str(&format!(" ({version})"));
            }

            if let Some(ref arch) = image.architecture {
                summary.push_str(&format!(" [{arch}]"));
            }

            summaries.push(summary);
        }

        summaries
    }

    /// 获取主要版本信息（如果有多个镜像，返回最常见的版本）
    pub fn get_primary_version(&self) -> Option<String> {
        if self.images.is_empty() {
            return None;
        }

        // 统计版本出现频率
        let mut version_counts = std::collections::HashMap::new();
        for image in &self.images {
            if let Some(ref version) = image.version {
                *version_counts.entry(version.clone()).or_insert(0) += 1;
            }
        }

        // 找到最常见的版本
        version_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(version, _)| version)
    }

    /// 获取主要架构信息（如果有多个镜像，返回最常见的架构）
    pub fn get_primary_architecture(&self) -> Option<String> {
        if self.images.is_empty() {
            return None;
        }

        // 统计架构出现频率
        let mut arch_counts = std::collections::HashMap::new();
        for image in &self.images {
            if let Some(ref arch) = image.architecture {
                *arch_counts.entry(arch.clone()).or_insert(0) += 1;
            }
        }

        // 找到最常见的架构
        arch_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(arch, _)| arch)
    }

    /// 检查是否包含指定版本的镜像
    #[allow(dead_code)]
    pub fn has_version(&self, version: &str) -> bool {
        self.images.iter().any(|img| {
            img.version
                .as_ref()
                .is_some_and(|v| v.to_lowercase().contains(&version.to_lowercase()))
        })
    }

    /// 检查是否包含指定架构的镜像
    #[allow(dead_code)]
    pub fn has_architecture(&self, arch: &str) -> bool {
        self.images.iter().any(|img| {
            img.architecture
                .as_ref()
                .is_some_and(|a| a.to_lowercase().contains(&arch.to_lowercase()))
        })
    }

    /// 获取Windows版本的详细信息
    pub fn get_windows_info(&self) -> Option<WindowsInfo> {
        let primary_version = self.get_primary_version()?;
        let primary_arch = self.get_primary_architecture()?;

        // 检查是否是Windows镜像
        if !primary_version.to_lowercase().contains("windows") {
            return None;
        }

        // 计算总的镜像版本（如Pro, Home, Enterprise等）
        let mut editions = Vec::new();
        for image in &self.images {
            let name_lower = image.name.to_lowercase();
            if name_lower.contains("pro") && !editions.contains(&"Pro".to_string()) {
                editions.push("Pro".to_string());
            } else if name_lower.contains("home") && !editions.contains(&"Home".to_string()) {
                editions.push("Home".to_string());
            } else if name_lower.contains("enterprise")
                && !editions.contains(&"Enterprise".to_string())
            {
                editions.push("Enterprise".to_string());
            } else if name_lower.contains("education")
                && !editions.contains(&"Education".to_string())
            {
                editions.push("Education".to_string());
            }
        }

        Some(WindowsInfo {
            version: primary_version,
            architecture: primary_arch,
            editions,
            image_count: self.images.len() as u32,
            total_size: self.images.iter().map(|img| img.total_bytes).sum(),
        })
    }
}

/// Windows 版本信息摘要
#[derive(Debug, Clone)]
pub struct WindowsInfo {
    pub version: String,
    pub architecture: String,
    pub editions: Vec<String>,
    pub image_count: u32,
    pub total_size: u64,
}

impl std::fmt::Display for WindowsInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.version, self.architecture)?;
        if !self.editions.is_empty() {
            write!(f, " - 版本: {}", self.editions.join(", "))?;
        }
        write!(f, " | 镜像数量: {}", self.image_count)?;
        write!(f, " | 总大小: {} MB", self.total_size / (1024 * 1024))?;
        Ok(())
    }
}
