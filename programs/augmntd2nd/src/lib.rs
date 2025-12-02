use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, MintTo};

declare_id!("GqVqdhyzJquzUzXoogf3GGeHFcoazt3ed4kb4x9aFukh");

#[program]
pub mod augmntd2nd {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, seed_id: u64, difficulty: u8) -> Result<()> {
        let seed_state = &mut ctx.accounts.seed_state;
        seed_state.seed_id = seed_id;
        seed_state.difficulty = difficulty;
        seed_state.total_bridges = 0;
        seed_state.pathway_a_count = 0;
        seed_state.pathway_b_count = 0;
        seed_state.pathway_c_count = 0;
        seed_state.is_active = true;
        
        // Set the fragment data as per the .ic file
        seed_state.fragment_data = "Interval: [C --3 semitones--> ?]".to_string();
        
        msg!("Seed 65 Initialized: The Enharmonic Gap is open.");
        Ok(())
    }

    pub fn bridge_enharmonic_gap(
        ctx: Context<BridgeGap>,
        completion: EnharmonicCompletion,
    ) -> Result<()> {
        // 1. Verification (Read-only)
        require!(ctx.accounts.seed_state.is_active, MEMEkError::SeedInactive);
        require!(completion.salt > 0, MEMEkError::InvalidProof);

        let coherence_score = verify_harmonic_coherence(
            &completion.context,
            &completion.interval_name,
            &completion.resolution
        )?;

        require!(
            coherence_score >= 70,
            MEMEkError::ContextualIncoherence
        );

        // 2. Determine pathway (Read-only logic)
        let (reward, pathway, pathway_type) = if completion.interval_name.to_lowercase().contains("augmented") {
            (65, "Pathway A: Augmented Second", 0)
        } else if completion.interval_name.to_lowercase().contains("minor") && !completion.interval_name.to_lowercase().contains("both") {
            (65, "Pathway B: Minor Third", 1)
        } else if completion.interval_name.to_lowercase().contains("both") || completion.interval_name.to_lowercase().contains("superposition") || completion.interval_name.to_lowercase().contains("janus") {
            (165, "Pathway C: Janus Mode", 2)
        } else {
            return Err(MEMEkError::UnrecognizedInterpretation.into());
        };

        // 3. Mint tokens (CPI)
        let seed_id = ctx.accounts.seed_state.seed_id;
        let seed_id_bytes = seed_id.to_le_bytes();
        let seeds = &[
            b"seed_state".as_ref(),
            seed_id_bytes.as_ref(),
            &[ctx.bumps.seed_state],
        ];
        let signer = &[&seeds[..]];

        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.memek_mint.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.seed_state.to_account_info(),
                },
                signer,
            ),
            reward,
        )?;

        // 4. Update State (Mutable borrow)
        let seed_state = &mut ctx.accounts.seed_state;
        match pathway_type {
            0 => seed_state.pathway_a_count += 1,
            1 => seed_state.pathway_b_count += 1,
            2 => seed_state.pathway_c_count += 1,
            _ => {},
        }
        seed_state.total_bridges += 1;

        msg!("Gap Bridged via {}! Reward: {} MEMEk", pathway, reward);
        msg!("Coherence Score: {}", coherence_score);

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(seed_id: u64)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 8 + 1 + 8 + 8 + 8 + 8 + 1 + 200, // Discriminator + fields + string buffer
        seeds = [b"seed_state", seed_id.to_le_bytes().as_ref()],
        bump
    )]
    pub seed_state: Account<'info, Seed65State>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BridgeGap<'info> {
    #[account(
        mut,
        seeds = [b"seed_state", seed_state.seed_id.to_le_bytes().as_ref()],
        bump
    )]
    pub seed_state: Account<'info, Seed65State>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub memek_mint: Account<'info, Mint>,
    
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct Seed65State {
    pub seed_id: u64,
    pub difficulty: u8,
    pub total_bridges: u64,
    pub pathway_a_count: u64,
    pub pathway_b_count: u64,
    pub pathway_c_count: u64,
    pub is_active: bool,
    pub fragment_data: String,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct EnharmonicCompletion {
    pub context: String,
    pub interval_name: String,
    pub resolution: String,
    pub salt: u64,
}

#[error_code]
pub enum MEMEkError {
    #[msg("Seed is currently inactive.")]
    SeedInactive,
    #[msg("Proof of Incompleteness failed (invalid hash/salt).")]
    InvalidProof,
    #[msg("Harmonic coherence score too low. Context does not justify the interval.")]
    ContextualIncoherence,
    #[msg("Unrecognized interval interpretation.")]
    UnrecognizedInterpretation,
}

// Helper function for semantic verification
fn verify_harmonic_coherence(
    context: &str,
    interval_name: &str,
    resolution: &str,
) -> Result<u8> {
    let mut score = 0u8;
    let context_lower = context.to_lowercase();
    let interval_lower = interval_name.to_lowercase();
    let resolution_lower = resolution.to_lowercase();

    // 1. Context Check
    // If context mentions "harmonic minor" and interval is "augmented", that's coherent.
    if context_lower.contains("harmonic minor") && interval_lower.contains("augmented") {
        score += 40;
    }
    // If context mentions "natural minor" or "major" and interval is "minor", that's coherent.
    else if (context_lower.contains("natural minor") || context_lower.contains("major") || context_lower.contains("c minor")) && interval_lower.contains("minor") {
        score += 40;
    }
    // Janus mode: Context mentions both or superposition
    else if (context_lower.contains("both") || context_lower.contains("superposition") || context_lower.contains("schrodinger")) && (interval_lower.contains("both") || interval_lower.contains("janus")) {
        score += 50; // Bonus for Janus
    }
    // Partial credit for just getting the interval name right in a generic context
    else if interval_lower.contains("augmented") || interval_lower.contains("minor") {
        score += 20;
    }

    // 2. Resolution Check
    // Augmented 2nd resolves outward/up (D# -> E)
    if interval_lower.contains("augmented") && (resolution_lower.contains("up") || resolution_lower.contains("outward") || resolution_lower.contains("e")) {
        score += 30;
    }
    // Minor 3rd is stable or moves stepwise
    else if interval_lower.contains("minor") && (resolution_lower.contains("stable") || resolution_lower.contains("step") || resolution_lower.contains("triad")) {
        score += 30;
    }
    // Janus resolution: "doesn't move" or "context shifts"
    else if interval_lower.contains("janus") || interval_lower.contains("both") {
        if resolution_lower.contains("shift") || resolution_lower.contains("context") || resolution_lower.contains("transform") {
            score += 30;
        }
    }

    // 3. Base score for effort (length of input)
    if context.len() > 5 && resolution.len() > 5 {
        score += 10;
    }

    Ok(score)
}
