//! MICRO WALLET ARCHITECTURE
//! 
//! Specialized wallet allocation system for micro-lightning operations
//! Implements $20/60min portfolio segmentation with tactical allocations

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};

/// Wallet types for micro-lightning operations
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum WalletType {
    Lightning,      // Primary trading wallet
    EmergencyGas,   // Emergency gas reserves
    Reentry,        // Re-entry buffer
    Psychology,     // Psychology fund (profit tax)
    TacticalExit,   // Tactical exit reserves
}

/// Micro wallet configuration for $20/60min operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroWallet {
    pub lightning: f64,       // $4.0 - Primary trading capital
    pub emergency_gas: f64,   // $3.5 - Emergency gas reserves
    pub reentry: f64,         // $4.5 - Re-entry buffer
    pub psychology: f64,      // $4.0 - Psychology fund
    pub tactical_exit: f64,   // $4.0 - Tactical exit reserves
    pub total_capital: f64,   // Total available capital
    pub allocation_ratios: HashMap<WalletType, f64>,
}

impl MicroWallet {
    /// Create new micro wallet with default $20 allocation
    pub fn new() -> Self {
        let mut allocation_ratios = HashMap::new();
        allocation_ratios.insert(WalletType::Lightning, 0.20);      // 20%
        allocation_ratios.insert(WalletType::EmergencyGas, 0.175);  // 17.5%
        allocation_ratios.insert(WalletType::Reentry, 0.225);       // 22.5%
        allocation_ratios.insert(WalletType::Psychology, 0.20);     // 20%
        allocation_ratios.insert(WalletType::TacticalExit, 0.20);   // 20%

        Self {
            lightning: 4.0,
            emergency_gas: 3.5,
            reentry: 4.5,
            psychology: 4.0,
            tactical_exit: 4.0,
            total_capital: 20.0,
            allocation_ratios,
        }
    }

    /// Create micro wallet with custom capital amount
    pub fn with_capital(total_capital: f64) -> Self {
        let mut wallet = Self::new();
        wallet.total_capital = total_capital;
        wallet.rebalance_allocations();
        wallet
    }

    /// Rebalance allocations based on total capital
    pub fn rebalance_allocations(&mut self) {
        self.lightning = self.total_capital * self.allocation_ratios[&WalletType::Lightning];
        self.emergency_gas = self.total_capital * self.allocation_ratios[&WalletType::EmergencyGas];
        self.reentry = self.total_capital * self.allocation_ratios[&WalletType::Reentry];
        self.psychology = self.total_capital * self.allocation_ratios[&WalletType::Psychology];
        self.tactical_exit = self.total_capital * self.allocation_ratios[&WalletType::TacticalExit];

        info!(
            "💰 Wallet rebalanced - Lightning: ${:.2}, Emergency: ${:.2}, Reentry: ${:.2}, Psychology: ${:.2}, Tactical: ${:.2}",
            self.lightning, self.emergency_gas, self.reentry, self.psychology, self.tactical_exit
        );
    }

    /// Get available balance for specific wallet type
    pub fn get_balance(&self, wallet_type: &WalletType) -> f64 {
        match wallet_type {
            WalletType::Lightning => self.lightning,
            WalletType::EmergencyGas => self.emergency_gas,
            WalletType::Reentry => self.reentry,
            WalletType::Psychology => self.psychology,
            WalletType::TacticalExit => self.tactical_exit,
        }
    }

    /// Allocate funds from specific wallet
    pub fn allocate_funds(&mut self, wallet_type: &WalletType, amount: f64) -> Result<f64> {
        let available = self.get_balance(wallet_type);
        
        if amount > available {
            warn!(
                "❌ Insufficient funds in {:?} wallet: requested ${:.2}, available ${:.2}",
                wallet_type, amount, available
            );
            return Ok(available); // Return maximum available
        }

        match wallet_type {
            WalletType::Lightning => self.lightning -= amount,
            WalletType::EmergencyGas => self.emergency_gas -= amount,
            WalletType::Reentry => self.reentry -= amount,
            WalletType::Psychology => self.psychology -= amount,
            WalletType::TacticalExit => self.tactical_exit -= amount,
        }

        info!(
            "💸 Allocated ${:.2} from {:?} wallet, remaining: ${:.2}",
            amount, wallet_type, self.get_balance(wallet_type)
        );

        Ok(amount)
    }

    /// Return funds to specific wallet
    pub fn return_funds(&mut self, wallet_type: &WalletType, amount: f64) {
        match wallet_type {
            WalletType::Lightning => self.lightning += amount,
            WalletType::EmergencyGas => self.emergency_gas += amount,
            WalletType::Reentry => self.reentry += amount,
            WalletType::Psychology => self.psychology += amount,
            WalletType::TacticalExit => self.tactical_exit += amount,
        }

        info!(
            "💰 Returned ${:.2} to {:?} wallet, new balance: ${:.2}",
            amount, wallet_type, self.get_balance(wallet_type)
        );
    }

    /// Apply psychology tax (10% of profits to psychology fund)
    pub fn apply_psychology_tax(&mut self, profit: f64) -> f64 {
        if profit > 0.0 {
            let tax = profit * 0.1;
            self.psychology += tax;
            
            info!(
                "🧠 Psychology tax applied: ${:.2} (10% of ${:.2} profit)",
                tax, profit
            );
            
            return profit - tax;
        }
        profit
    }

    /// Get position sizing for lightning wallet
    pub fn get_lightning_position_size(&self, risk_percentage: f64) -> f64 {
        let position_size = self.lightning * risk_percentage.min(0.8); // Max 80% of lightning wallet
        
        info!(
            "⚡ Lightning position size: ${:.2} ({:.1}% of ${:.2})",
            position_size, risk_percentage * 100.0, self.lightning
        );
        
        position_size
    }

    /// Get reentry allocation
    pub fn get_reentry_allocation(&self, boost_percentage: f64) -> f64 {
        let allocation = self.reentry * boost_percentage.min(0.6); // Max 60% of reentry buffer
        
        info!(
            "🔄 Reentry allocation: ${:.2} ({:.1}% of ${:.2})",
            allocation, boost_percentage * 100.0, self.reentry
        );
        
        allocation
    }

    /// Get tactical exit allocation for DLMM positions
    pub fn get_tactical_exit_allocation(&self) -> f64 {
        let allocation = self.tactical_exit * 0.375; // 37.5% for DLMM
        
        info!(
            "🎯 Tactical exit allocation: ${:.2} (37.5% of ${:.2})",
            allocation, self.tactical_exit
        );
        
        allocation
    }

    /// Check if emergency gas is sufficient
    pub fn has_sufficient_emergency_gas(&self, required_gas: f64) -> bool {
        let sufficient = self.emergency_gas >= required_gas;
        
        if !sufficient {
            warn!(
                "⛽ Insufficient emergency gas: required ${:.2}, available ${:.2}",
                required_gas, self.emergency_gas
            );
        }
        
        sufficient
    }

    /// Get wallet utilization summary
    pub fn get_utilization_summary(&self) -> WalletUtilization {
        let total_allocated = self.lightning + self.emergency_gas + self.reentry + self.psychology + self.tactical_exit;
        let utilization_rate = total_allocated / self.total_capital;

        WalletUtilization {
            total_capital: self.total_capital,
            total_allocated,
            utilization_rate,
            lightning_ratio: self.lightning / self.total_capital,
            emergency_ratio: self.emergency_gas / self.total_capital,
            reentry_ratio: self.reentry / self.total_capital,
            psychology_ratio: self.psychology / self.total_capital,
            tactical_ratio: self.tactical_exit / self.total_capital,
        }
    }

    /// Reset wallet to initial state (for rotation)
    pub fn reset_for_rotation(&mut self) {
        info!("🔄 Resetting wallet for rotation");
        
        // Keep psychology fund, reset others
        let psychology_balance = self.psychology;
        *self = Self::with_capital(self.total_capital);
        self.psychology = psychology_balance;
        
        info!("✅ Wallet reset complete, psychology fund preserved: ${:.2}", psychology_balance);
    }

    /// Validate wallet integrity
    pub fn validate_integrity(&self) -> Result<()> {
        let total_allocated = self.lightning + self.emergency_gas + self.reentry + self.psychology + self.tactical_exit;
        let tolerance = 0.01; // $0.01 tolerance for floating point precision
        
        if (total_allocated - self.total_capital).abs() > tolerance {
            return Err(anyhow::anyhow!(
                "Wallet integrity check failed: allocated ${:.2}, total ${:.2}",
                total_allocated, self.total_capital
            ));
        }
        
        Ok(())
    }
}

impl Default for MicroWallet {
    fn default() -> Self {
        Self::new()
    }
}

/// Wallet utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletUtilization {
    pub total_capital: f64,
    pub total_allocated: f64,
    pub utilization_rate: f64,
    pub lightning_ratio: f64,
    pub emergency_ratio: f64,
    pub reentry_ratio: f64,
    pub psychology_ratio: f64,
    pub tactical_ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_micro_wallet_creation() {
        let wallet = MicroWallet::new();
        assert_eq!(wallet.total_capital, 20.0);
        assert_eq!(wallet.lightning, 4.0);
        assert_eq!(wallet.emergency_gas, 3.5);
        assert_eq!(wallet.reentry, 4.5);
        assert_eq!(wallet.psychology, 4.0);
        assert_eq!(wallet.tactical_exit, 4.0);
    }

    #[test]
    fn test_wallet_allocation() {
        let mut wallet = MicroWallet::new();
        let allocated = wallet.allocate_funds(&WalletType::Lightning, 2.0).unwrap();
        assert_eq!(allocated, 2.0);
        assert_eq!(wallet.lightning, 2.0);
    }

    #[test]
    fn test_psychology_tax() {
        let mut wallet = MicroWallet::new();
        let after_tax = wallet.apply_psychology_tax(10.0);
        assert_eq!(after_tax, 9.0);
        assert_eq!(wallet.psychology, 5.0); // 4.0 + 1.0 tax
    }

    #[test]
    fn test_wallet_integrity() {
        let wallet = MicroWallet::new();
        assert!(wallet.validate_integrity().is_ok());
    }
}
