//! FORGE Test Harness - Dynamic Strategy Loading Tester
//! 
//! Test harness dla weryfikacji skompilowanych strategii DSL
//! Dynamiczne ładowanie i testowanie .so artifacts

use anyhow::{Result, anyhow};
use libloading::{Library, Symbol};
use std::env;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::Path;

/// Strategy VTable - ABI interface (musi być identyczna z hot_loader.rs)
#[repr(C)]
#[derive(Debug)]
pub struct StrategyVTable {
    pub analyze: unsafe extern "C" fn(*const MarketData) -> f64,
    pub execute: unsafe extern "C" fn(*mut HftContext) -> i32,
    pub cleanup: unsafe extern "C" fn(),
    pub get_info: unsafe extern "C" fn() -> *const StrategyInfo,
    pub health_check: unsafe extern "C" fn() -> i32,
}

/// Market data structure (C-compatible)
#[repr(C)]
#[derive(Debug)]
pub struct MarketData {
    pub timestamp: u64,
    pub price: f64,
    pub volume: f64,
    pub bid: f64,
    pub ask: f64,
    pub momentum_signal: f64,
    pub volatility: f64,
    pub liquidity_score: f64,
}

/// HFT execution context (C-compatible)
#[repr(C)]
#[derive(Debug)]
pub struct HftContext {
    pub agent_id: *const c_char,
    pub position_size: f64,
    pub available_balance: f64,
    pub max_position_size: f64,
    pub risk_limit: f64,
    pub execution_callback: Option<unsafe extern "C" fn(*const c_char, f64, f64) -> i32>,
}

/// Strategy info structure (C-compatible)
#[repr(C)]
#[derive(Debug)]
pub struct StrategyInfo {
    pub name: *const c_char,
    pub version: *const c_char,
    pub author: *const c_char,
    pub description: *const c_char,
    pub risk_level: u8,
    pub expected_return: f64,
    pub max_drawdown: f64,
}

/// Test results
#[derive(Debug)]
pub struct TestResults {
    pub library_loaded: bool,
    pub symbols_found: bool,
    pub health_check_passed: bool,
    pub analyze_test_passed: bool,
    pub execute_test_passed: bool,
    pub info_test_passed: bool,
    pub cleanup_successful: bool,
    pub overall_success: bool,
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <strategy.so>", args[0]);
        std::process::exit(1);
    }
    
    let strategy_path = &args[1];
    
    println!("🔥 FORGE Test Harness - Strategy Verification");
    println!("📁 Testing strategy: {}", strategy_path);
    println!("{}", "=".repeat(60));
    
    let results = test_strategy(strategy_path)?;
    
    println!("\n📊 TEST RESULTS:");
    println!("{}", "=".repeat(60));
    println!("📦 Library Loaded:      {}", if results.library_loaded { "✅ PASS" } else { "❌ FAIL" });
    println!("🔗 Symbols Found:       {}", if results.symbols_found { "✅ PASS" } else { "❌ FAIL" });
    println!("💓 Health Check:        {}", if results.health_check_passed { "✅ PASS" } else { "❌ FAIL" });
    println!("📊 Analyze Function:    {}", if results.analyze_test_passed { "✅ PASS" } else { "❌ FAIL" });
    println!("⚡ Execute Function:    {}", if results.execute_test_passed { "✅ PASS" } else { "❌ FAIL" });
    println!("ℹ️  Info Function:       {}", if results.info_test_passed { "✅ PASS" } else { "❌ FAIL" });
    println!("🧹 Cleanup:             {}", if results.cleanup_successful { "✅ PASS" } else { "❌ FAIL" });
    println!("{}", "=".repeat(60));
    println!("🎯 OVERALL:             {}", if results.overall_success { "✅ SUCCESS" } else { "❌ FAILURE" });
    
    if results.overall_success {
        println!("\n🎉 Strategy artifact is COMBAT READY!");
        std::process::exit(0);
    } else {
        println!("\n💥 Strategy artifact FAILED verification!");
        std::process::exit(1);
    }
}

fn test_strategy(strategy_path: &str) -> Result<TestResults> {
    let mut results = TestResults {
        library_loaded: false,
        symbols_found: false,
        health_check_passed: false,
        analyze_test_passed: false,
        execute_test_passed: false,
        info_test_passed: false,
        cleanup_successful: false,
        overall_success: false,
    };
    
    // Test 1: Load library
    println!("🔍 Test 1: Loading strategy library...");
    
    if !Path::new(strategy_path).exists() {
        return Err(anyhow!("Strategy file not found: {}", strategy_path));
    }
    
    let library = unsafe {
        match Library::new(strategy_path) {
            Ok(lib) => {
                println!("   ✅ Library loaded successfully");
                results.library_loaded = true;
                lib
            }
            Err(e) => {
                println!("   ❌ Failed to load library: {}", e);
                return Ok(results);
            }
        }
    };
    
    // Test 2: Extract symbols
    println!("🔍 Test 2: Extracting strategy symbols...");
    
    let vtable = match extract_vtable(&library) {
        Ok(vt) => {
            println!("   ✅ All required symbols found");
            results.symbols_found = true;
            vt
        }
        Err(e) => {
            println!("   ❌ Failed to extract symbols: {}", e);
            return Ok(results);
        }
    };
    
    // Test 3: Health check
    println!("🔍 Test 3: Strategy health check...");
    
    let health_status = unsafe { (vtable.health_check)() };
    match health_status {
        0 => {
            println!("   ✅ Health check passed (status: {})", health_status);
            results.health_check_passed = true;
        }
        1 => {
            println!("   ⚠️  Health check warning (status: {})", health_status);
            results.health_check_passed = true; // Warning is acceptable
        }
        _ => {
            println!("   ❌ Health check failed (status: {})", health_status);
            return Ok(results);
        }
    }
    
    // Test 4: Analyze function
    println!("🔍 Test 4: Testing analyze function...");
    
    let test_market_data = MarketData {
        timestamp: 1640995200, // 2022-01-01
        price: 100.0,
        volume: 1000000.0,
        bid: 99.5,
        ask: 100.5,
        momentum_signal: 0.5,
        volatility: 0.02,
        liquidity_score: 0.8,
    };
    
    let signal_strength = unsafe { (vtable.analyze)(&test_market_data as *const MarketData) };
    
    if signal_strength >= -1.0 && signal_strength <= 1.0 {
        println!("   ✅ Analyze function returned valid signal: {:.3}", signal_strength);
        results.analyze_test_passed = true;
    } else {
        println!("   ❌ Analyze function returned invalid signal: {:.3}", signal_strength);
        return Ok(results);
    }
    
    // Test 5: Execute function
    println!("🔍 Test 5: Testing execute function...");
    
    let agent_id = CString::new("test_agent").unwrap();
    let mut test_context = HftContext {
        agent_id: agent_id.as_ptr(),
        position_size: 1000.0,
        available_balance: 10000.0,
        max_position_size: 5000.0,
        risk_limit: 0.05,
        execution_callback: Some(mock_execution_callback),
    };
    
    let execute_result = unsafe { (vtable.execute)(&mut test_context as *mut HftContext) };
    
    if execute_result >= 0 {
        println!("   ✅ Execute function completed successfully (result: {})", execute_result);
        results.execute_test_passed = true;
    } else {
        println!("   ❌ Execute function failed (result: {})", execute_result);
        return Ok(results);
    }
    
    // Test 6: Info function
    println!("🔍 Test 6: Testing info function...");
    
    let info_ptr = unsafe { (vtable.get_info)() };
    
    if info_ptr.is_null() {
        println!("   ❌ Info function returned null pointer");
        return Ok(results);
    }
    
    unsafe {
        let info = &*info_ptr;
        
        if info.name.is_null() || info.version.is_null() {
            println!("   ❌ Info structure contains null pointers");
            return Ok(results);
        }
        
        let name = CStr::from_ptr(info.name).to_string_lossy();
        let version = CStr::from_ptr(info.version).to_string_lossy();
        let description = if info.description.is_null() {
            "N/A".to_string()
        } else {
            CStr::from_ptr(info.description).to_string_lossy().to_string()
        };
        
        println!("   ✅ Strategy Info:");
        println!("      📛 Name: {}", name);
        println!("      🔢 Version: {}", version);
        println!("      📝 Description: {}", description);
        println!("      ⚠️  Risk Level: {}", info.risk_level);
        println!("      📈 Expected Return: {:.2}%", info.expected_return * 100.0);
        println!("      📉 Max Drawdown: {:.2}%", info.max_drawdown * 100.0);
        
        results.info_test_passed = true;
    }
    
    // Test 7: Cleanup
    println!("🔍 Test 7: Testing cleanup function...");
    
    unsafe {
        (vtable.cleanup)();
        println!("   ✅ Cleanup function executed successfully");
        results.cleanup_successful = true;
    }
    
    // Overall success
    results.overall_success = results.library_loaded
        && results.symbols_found
        && results.health_check_passed
        && results.analyze_test_passed
        && results.execute_test_passed
        && results.info_test_passed
        && results.cleanup_successful;
    
    Ok(results)
}

fn extract_vtable(library: &Library) -> Result<StrategyVTable> {
    unsafe {
        let analyze: Symbol<unsafe extern "C" fn(*const MarketData) -> f64> = 
            library.get(b"strategy_analyze")
                .map_err(|e| anyhow!("Failed to load analyze function: {}", e))?;
        
        let execute: Symbol<unsafe extern "C" fn(*mut HftContext) -> i32> = 
            library.get(b"strategy_execute")
                .map_err(|e| anyhow!("Failed to load execute function: {}", e))?;
        
        let cleanup: Symbol<unsafe extern "C" fn()> = 
            library.get(b"strategy_cleanup")
                .map_err(|e| anyhow!("Failed to load cleanup function: {}", e))?;
        
        let get_info: Symbol<unsafe extern "C" fn() -> *const StrategyInfo> = 
            library.get(b"strategy_get_info")
                .map_err(|e| anyhow!("Failed to load get_info function: {}", e))?;
        
        let health_check: Symbol<unsafe extern "C" fn() -> i32> = 
            library.get(b"strategy_health_check")
                .map_err(|e| anyhow!("Failed to load health_check function: {}", e))?;
        
        Ok(StrategyVTable {
            analyze: *analyze,
            execute: *execute,
            cleanup: *cleanup,
            get_info: *get_info,
            health_check: *health_check,
        })
    }
}

/// Mock execution callback for testing
unsafe extern "C" fn mock_execution_callback(
    agent_id: *const c_char,
    position_size: f64,
    price: f64,
) -> i32 {
    if agent_id.is_null() {
        return -1;
    }
    
    let agent_id_str = CStr::from_ptr(agent_id).to_string_lossy();
    println!("   📞 Mock execution callback: agent={}, size={:.2}, price={:.2}", 
             agent_id_str, position_size, price);
    
    // Simulate successful execution
    0
}
