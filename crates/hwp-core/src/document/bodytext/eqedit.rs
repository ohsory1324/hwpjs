/// EqEdit 구조체 / EqEdit structure
///
/// 스펙 문서 매핑: 표 105 - 수식 개체 속성 / Spec mapping: Table 105 - Equation editor object attributes
///
/// **구현 상태 / Implementation Status**
/// - 구현 완료 / Implementation complete
/// - 테스트 파일(`noori.hwp`)에 EQEDIT 레코드가 없어 실제 파일로 테스트되지 않음
/// - Implementation complete, but not tested with actual file as test file (`noori.hwp`) does not contain EQEDIT records
use crate::error::HwpError;
use crate::types::{COLORREF, HWPUNIT, INT16, UINT16, UINT32};
use serde::{Deserialize, Serialize};

/// 수식 개체 / Equation editor object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqEdit {
    /// 속성. 스크립트가 차지하는 범위. 첫 비트가 켜져 있으면 줄 단위, 꺼져 있으면 글자 단위.
    /// Attribute. Range occupied by script. If first bit is set, line unit; otherwise, character unit.
    pub attribute: UINT32,
    /// 스크립트 길이 / Script length
    pub script_length: UINT16,
    /// 한글 수식 스크립트 / HWP equation script (EQN script compatible)
    pub script: String,
    /// 수식 글자 크기 / Equation character size
    pub character_size: HWPUNIT,
    /// 글자 색상 / Character color
    pub character_color: COLORREF,
    /// base line / Base line
    pub base_line: INT16,
    /// 수식 버전 정보 / Equation version information
    pub version_info: String,
    /// 수식 폰트 이름 / Equation font name
    pub font_name: String,
}

impl EqEdit {
    /// EqEdit을 바이트 배열에서 파싱합니다. / Parse EqEdit from byte array.
    ///
    /// # Arguments
    /// * `data` - EqEdit 데이터 (수식 개체 속성 부분만) / EqEdit data (equation editor object attributes only)
    ///
    /// # Returns
    /// 파싱된 EqEdit 구조체 / Parsed EqEdit structure
    ///
    /// # Note
    /// 스펙 문서 표 104에 따르면 EQEDIT는 다음 구조를 가집니다:
    /// - 개체 공통 속성(표 68 참조) - 가변 길이
    /// - 수식 개체 속성(표 105 참조) - 가변 길이 (16 + 6×len 바이트)
    ///
    /// 레거시 코드(hwp.js)는 수식 개체 속성의 일부만 파싱하고 있습니다.
    /// According to spec Table 104, EQEDIT has the following structure:
    /// - Object common properties (Table 68) - variable length
    /// - Equation editor object attributes (Table 105) - variable length (16 + 6×len bytes)
    ///
    /// Legacy code (hwp.js) only parses part of equation editor object attributes.
    ///
    /// 실제 문서의 레코드는 스펙 표기(16 + 6×len)와 다릅니다. 아래 본문 주석 참고.
    /// Real records differ from the spec's 16 + 6×len layout; see comments below.
    pub fn parse(data: &[u8]) -> Result<Self, HwpError> {
        // 실제 레코드 배치 (HWP 5.0.0.6~5.1.0.1 실제 문서의 EQEDIT 레코드 실측):
        //   UINT32 속성, WORD len + WCHAR[len] 스크립트, HWPUNIT 글자 크기, COLORREF 색상,
        //   INT16 base line, 2바이트(스펙에 없음, 실측상 항상 존재),
        //   WORD len + WCHAR[len] 버전 정보, WORD len + WCHAR[len] 폰트 이름.
        // 스펙 표 105는 버전 정보·폰트 이름을 스크립트와 같은 len으로 적었지만(16+6×len)
        // 실제로는 각자 길이 접두가 붙은 문자열이고, 옛 판(5.0.0.x/5.0.2.x)은 뒤 문자열이 아예 없습니다.
        // 그래서 속성과 스크립트 길이(6바이트)만 요구하고 나머지는 있는 만큼만 읽습니다.
        // Actual layout: version info and font name are length-prefixed strings (not the script
        // len), preceded by 2 undocumented bytes; older files omit them. Read leniently.
        if data.len() < 6 {
            return Err(HwpError::insufficient_data("EqEdit", 6, data.len()));
        }

        let mut reader = LenientReader { data, offset: 0 };

        // 표 105: 속성 (UINT32) / Table 105: Attribute (UINT32)
        let attribute = reader.u32().unwrap_or(0);
        // 표 105: 스크립트 길이 (WORD) + 스크립트 (WCHAR[len])
        let script_length = reader.u16().unwrap_or(0);
        let script = reader.wchars(script_length as usize);
        // 표 105: 수식 글자 크기 (HWPUNIT) / Equation character size
        let character_size = HWPUNIT::from(reader.u32().unwrap_or(0));
        // 표 105: 글자 색상 (COLORREF) / Character color
        let character_color = COLORREF(reader.u32().unwrap_or(0));
        // 표 105: base line (INT16)
        let base_line = reader.u16().map(|v| v as INT16).unwrap_or(0);
        // 스펙에 없는 2바이트 / 2 undocumented bytes
        let _ = reader.u16();
        // 수식 버전 정보 (WORD len + WCHAR[len]) / Equation version information
        let version_info = reader.bstr();
        // 수식 폰트 이름 (WORD len + WCHAR[len]) / Equation font name
        let font_name = reader.bstr();

        Ok(EqEdit {
            attribute,
            script_length,
            script,
            character_size,
            character_color,
            base_line,
            version_info,
            font_name,
        })
    }

}

/// 남은 바이트가 모자라면 None/빈 문자열을 돌려주는 읽기 도우미 / Reader that never panics on short data
struct LenientReader<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> LenientReader<'a> {
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        if self.offset + n <= self.data.len() {
            let slice = &self.data[self.offset..self.offset + n];
            self.offset += n;
            Some(slice)
        } else {
            None
        }
    }

    fn u16(&mut self) -> Option<UINT16> {
        self.take(2).map(|b| UINT16::from_le_bytes([b[0], b[1]]))
    }

    fn u32(&mut self) -> Option<UINT32> {
        self.take(4)
            .map(|b| UINT32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    /// WCHAR[len]을 읽습니다. 모자라면 남은 온전한 글자까지만 읽습니다.
    /// Reads WCHAR[len]; if short, reads only the complete characters that remain.
    fn wchars(&mut self, len: usize) -> String {
        let available = (self.data.len() - self.offset) / 2;
        let n = len.min(available);
        let bytes = self.take(2 * n).unwrap_or(&[]);
        let units: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        String::from_utf16_lossy(&units)
    }

    /// WORD len + WCHAR[len] 문자열 / Length-prefixed WCHAR string
    fn bstr(&mut self) -> String {
        match self.u16() {
            Some(len) => self.wchars(len as usize),
            None => String::new(),
        }
    }
}
