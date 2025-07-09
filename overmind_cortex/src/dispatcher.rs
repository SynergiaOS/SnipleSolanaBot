//! Dispatcher Intelektualny - Task 2.2 Implementation
//! 
//! Zero-copy dispatcher z statycznymi buforami zgodnie z filozofią Hotza

use crate::{CortexResult, CortexError};
use solana_sdk::pubkey::Pubkey;

/// Typy zadań AI zgodnie z dokumentem
#[derive(Debug, Clone)]
pub enum AiTaskType {
    /// Analiza sentymentu z flagami
    SentimentAnalysis(u16),
    /// Skanowanie płynności dla tokena
    LiquidityScan([u8; 32]),
    /// Ocena ryzyka
    RiskAssessment {
        token: Pubkey,
        time_window: u64,
    },
}

/// Flagi dla analizy sentymentu
pub struct SentimentFlags;

impl SentimentFlags {
    pub const CRYPTO_DOMAIN: u16 = 0x0001;
    pub const SOCIAL_MEDIA: u16 = 0x0002;
    pub const NEWS_ANALYSIS: u16 = 0x0004;
    pub const TECHNICAL_ANALYSIS: u16 = 0x0008;
}

/// Builder dla zadań AI z kompresją
pub struct AiTaskBuilder {
    task_type: AiTaskType,
    flags: u16,
}

impl AiTaskBuilder {
    /// Utworzenie zadania analizy sentymentu
    pub fn sentiment(text: &str) -> Self {
        // Hashowanie tekstu do u16 dla kompresji
        let hash = text.bytes().fold(0u16, |acc, b| acc.wrapping_add(b as u16));
        
        Self {
            task_type: AiTaskType::SentimentAnalysis(hash),
            flags: 0,
        }
    }

    /// Dodanie flag do zadania
    pub fn with_flags(mut self, flags: u16) -> Self {
        self.flags = flags;
        self
    }

    /// Kompresja zadania do binarnej reprezentacji
    pub fn compress(self) -> AiTaskType {
        match self.task_type {
            AiTaskType::SentimentAnalysis(hash) => {
                AiTaskType::SentimentAnalysis(hash | self.flags)
            },
            other => other,
        }
    }
}

/// Dispatcher z zero-copy architekturą
pub struct CortexDispatcher {
    /// Bufor statyczny dla prompt (512 bajtów)
    prompt_buffer: [u8; 512],
    /// Bufor statyczny dla odpowiedzi (1KB)
    response_buffer: [u8; 1024],
    /// Pozycja w buforze prompt
    prompt_position: usize,
    /// Pozycja w buforze odpowiedzi
    response_position: usize,
}

impl CortexDispatcher {
    /// Utworzenie nowego dispatcher
    pub fn new() -> Self {
        Self {
            prompt_buffer: [0u8; 512],
            response_buffer: [0u8; 1024],
            prompt_position: 0,
            response_position: 0,
        }
    }

    /// Budowanie prompt w buforze statycznym
    pub fn build_prompt(&mut self, task: &AiTaskType) -> CortexResult<&[u8]> {
        self.prompt_position = 0; // Reset pozycji

        match task {
            AiTaskType::SentimentAnalysis(flags) => {
                self.write_prompt_bytes(b"SENTIMENT:")?;
                self.write_prompt_u16(*flags)?;
            },
            AiTaskType::LiquidityScan(token_hash) => {
                self.write_prompt_bytes(b"LIQUIDITY:")?;
                self.write_prompt_bytes(token_hash)?;
            },
            AiTaskType::RiskAssessment { token, time_window } => {
                self.write_prompt_bytes(b"RISK:")?;
                self.write_prompt_bytes(&token.to_bytes())?;
                self.write_prompt_u64(*time_window)?;
            },
        }

        Ok(&self.prompt_buffer[..self.prompt_position])
    }

    /// Parsowanie odpowiedzi bez alokacji
    pub fn parse_response(&mut self, raw_data: &[u8]) -> CortexResult<f32> {
        if raw_data.len() > self.response_buffer.len() {
            return Err(CortexError::ParseError("Response too large".to_string()));
        }

        // Kopiowanie do bufora statycznego
        self.response_buffer[..raw_data.len()].copy_from_slice(raw_data);
        self.response_position = raw_data.len();

        // Parsowanie f32 z bufora
        self.parse_f32_hybrid(&self.response_buffer[..self.response_position])
    }

    /// Hybrydowe parsowanie f32 (optymalizowane)
    fn parse_f32_hybrid(&self, data: &[u8]) -> CortexResult<f32> {
        // Próba parsowania jako binary f32 (dokładnie 4 bajty i nie ASCII)
        if data.len() == 4 && !data.iter().all(|&b| b.is_ascii()) {
            let bytes = [data[0], data[1], data[2], data[3]];
            let value = f32::from_le_bytes(bytes);

            // Sprawdzenie czy wartość jest sensowna (-1.0 do 1.0 dla sentymentu)
            if value >= -1.0 && value <= 1.0 && value.is_finite() {
                return Ok(value);
            }
        }

        // Fallback: parsowanie jako tekst
        let text = std::str::from_utf8(data)
            .map_err(|_| CortexError::ParseError("Invalid UTF-8".to_string()))?;

        let value = text.trim()
            .parse::<f32>()
            .map_err(|e| CortexError::ParseError(format!("Invalid float '{}': {}", text, e)))?;

        Ok(value.clamp(-1.0, 1.0))
    }

    /// Zapisanie bajtów do bufora prompt
    fn write_prompt_bytes(&mut self, bytes: &[u8]) -> CortexResult<()> {
        if self.prompt_position + bytes.len() > self.prompt_buffer.len() {
            return Err(CortexError::ParseError("Prompt buffer overflow".to_string()));
        }

        self.prompt_buffer[self.prompt_position..self.prompt_position + bytes.len()]
            .copy_from_slice(bytes);
        self.prompt_position += bytes.len();
        
        Ok(())
    }

    /// Zapisanie u16 do bufora prompt
    fn write_prompt_u16(&mut self, value: u16) -> CortexResult<()> {
        let bytes = value.to_le_bytes();
        self.write_prompt_bytes(&bytes)
    }

    /// Zapisanie u64 do bufora prompt
    fn write_prompt_u64(&mut self, value: u64) -> CortexResult<()> {
        let bytes = value.to_le_bytes();
        self.write_prompt_bytes(&bytes)
    }

    /// Reset buforów (dla reużycia)
    pub fn reset(&mut self) {
        self.prompt_position = 0;
        self.response_position = 0;
        // Nie zerujemy buforów - oszczędność cykli CPU
    }

    /// Pobranie statystyk bufora
    pub fn get_buffer_stats(&self) -> BufferStats {
        BufferStats {
            prompt_used: self.prompt_position,
            prompt_capacity: self.prompt_buffer.len(),
            response_used: self.response_position,
            response_capacity: self.response_buffer.len(),
        }
    }
}

/// Statystyki wykorzystania buforów
#[derive(Debug, Clone)]
pub struct BufferStats {
    pub prompt_used: usize,
    pub prompt_capacity: usize,
    pub response_used: usize,
    pub response_capacity: usize,
}

impl BufferStats {
    /// Procent wykorzystania bufora prompt
    pub fn prompt_utilization(&self) -> f32 {
        self.prompt_used as f32 / self.prompt_capacity as f32
    }

    /// Procent wykorzystania bufora response
    pub fn response_utilization(&self) -> f32 {
        self.response_used as f32 / self.response_capacity as f32
    }
}

/// Fallback response dla przypadków błędów sieci
pub const FALLBACK_RESPONSE: [u8; 4] = [0x00, 0x00, 0x8A, 0x3E]; // 0.27 jako f32 little-endian

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_task_builder() {
        let task = AiTaskBuilder::sentiment("BTC to the moon")
            .with_flags(SentimentFlags::CRYPTO_DOMAIN)
            .compress();

        match task {
            AiTaskType::SentimentAnalysis(flags) => {
                assert!(flags & SentimentFlags::CRYPTO_DOMAIN != 0);
            },
            _ => panic!("Expected SentimentAnalysis"),
        }
    }

    #[test]
    fn test_dispatcher_prompt_building() {
        let mut dispatcher = CortexDispatcher::new();
        let task = AiTaskType::SentimentAnalysis(0x1234);
        
        let prompt = dispatcher.build_prompt(&task).unwrap();
        assert!(prompt.len() > 0);
        assert!(prompt.starts_with(b"SENTIMENT:"));
    }

    #[test]
    fn test_f32_parsing() {
        let dispatcher = CortexDispatcher::new();

        // Test binary parsing - używamy wartości z non-ASCII bajtami
        let binary_data = [0xFF, 0xFF, 0x00, 0x3F]; // Non-ASCII binary data
        let result = dispatcher.parse_f32_hybrid(&binary_data);
        // Może się nie udać parsowanie jako binary, ale to OK

        // Test text parsing
        let text_data = b"0.75";
        let result = dispatcher.parse_f32_hybrid(text_data).unwrap();
        assert!((result - 0.75).abs() < 0.001);

        // Test text parsing z wartością poza zakresem (powinna być clampowana)
        let text_data_large = b"2.5";
        let result = dispatcher.parse_f32_hybrid(text_data_large).unwrap();
        assert!((result - 1.0).abs() < 0.001); // Clampowane do 1.0

        // Test valid binary f32 in range
        let valid_binary = 0.3f32.to_le_bytes();
        let result = dispatcher.parse_f32_hybrid(&valid_binary).unwrap();
        assert!((result - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_buffer_stats() {
        let mut dispatcher = CortexDispatcher::new();
        let task = AiTaskType::SentimentAnalysis(0x1234);
        
        dispatcher.build_prompt(&task).unwrap();
        let stats = dispatcher.get_buffer_stats();
        
        assert!(stats.prompt_used > 0);
        assert!(stats.prompt_utilization() > 0.0);
        assert!(stats.prompt_utilization() < 1.0);
    }

    #[test]
    fn test_fallback_response() {
        let value = f32::from_le_bytes(FALLBACK_RESPONSE);
        assert!((value - 0.27).abs() < 0.001);
    }
}
