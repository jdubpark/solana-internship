use std::sync::Arc;

use anchor_lang::prelude::{ AccountMeta, Pubkey };

use gpl_nft_voter::state::max_voter_weight_record::{
    get_max_voter_weight_record_address,
    MaxVoterWeightRecord,
};
use gpl_nft_voter::state::*;

use spl_governance::instruction::cast_vote;
use spl_governance::state::vote_record::{ self, Vote, VoteChoice };

use gpl_nft_voter::state::{
    get_nft_vote_record_address,
    get_registrar_address,
    CollectionConfig,
    NftVoteRecord,
    Registrar,
    CompressedNftAsset as LeafVerificationCookie,
};

use solana_program_test::{ BanksClientError, ProgramTest };
use solana_sdk::instruction::Instruction;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use crate::program_test::governance_test::GovernanceTest;
use crate::program_test::program_test_bench::ProgramTestBench;
use crate::program_test::merkle_tree_test::{ LeafArgs, MerkleTreeTest };
use crate::program_test::governance_test::{ ProposalCookie, RealmCookie, TokenOwnerRecordCookie };
use crate::program_test::program_test_bench::WalletCookie;
use crate::program_test::token_metadata_test::{ NftCollectionCookie, NftCookie, TokenMetadataTest };
use crate::program_test::tools::NopOverride;

#[derive(Debug, PartialEq)]
pub struct RegistrarCookie {
    pub address: Pubkey,
    pub account: Registrar,

    pub realm_authority: Keypair,
    pub max_collections: u8,
}

pub struct VoterWeightRecordCookie {
    pub address: Pubkey,
    pub account: VoterWeightRecord,
}

pub struct MaxVoterWeightRecordCookie {
    pub address: Pubkey,
    pub account: MaxVoterWeightRecord,
}

pub struct CollectionConfigCookie {
    pub collection_config: CollectionConfig,
}

pub struct ConfigureCollectionArgs {
    pub weight: u64,
    pub size: u32,
}

impl Default for ConfigureCollectionArgs {
    fn default() -> Self {
        Self { weight: 1, size: 3 }
    }
}

#[derive(Debug, PartialEq)]
pub struct NftVoteRecordCookie {
    pub address: Pubkey,
    pub account: NftVoteRecord,
}

pub struct CastNftVoteArgs {
    pub cast_spl_gov_vote: bool,
}

impl Default for CastNftVoteArgs {
    fn default() -> Self {
        Self {
            cast_spl_gov_vote: true,
        }
    }
}

pub struct NftVoteTicketCookie {
    pub address: Pubkey,
    pub nft_mint: Pubkey,
}

pub struct NftVoterTest {
    pub program_id: Pubkey,
    pub bench: Arc<ProgramTestBench>,
    pub governance: GovernanceTest,
    pub token_metadata: TokenMetadataTest,
    pub merkle_tree: MerkleTreeTest,
}

impl NftVoterTest {
    #[allow(dead_code)]
    pub fn add_program(program_test: &mut ProgramTest) {
        program_test.add_program("gpl_nft_voter", gpl_nft_voter::id(), None);
    }

    #[allow(dead_code)]
    pub async fn start_new() -> Self {
        let mut program_test = ProgramTest::default();

        NftVoterTest::add_program(&mut program_test);
        GovernanceTest::add_program(&mut program_test);
        TokenMetadataTest::add_program(&mut program_test);
        MerkleTreeTest::add_program(&mut program_test);

        let program_id = gpl_nft_voter::id();

        let bench = ProgramTestBench::start_new(program_test).await;
        let bench_rc = Arc::new(bench);

        let governance_bench = GovernanceTest::new(
            bench_rc.clone(),
            Some(program_id),
            Some(program_id)
        );
        let token_metadata_bench = TokenMetadataTest::new(bench_rc.clone());
        let merkle_tree_bench = MerkleTreeTest::new(bench_rc.clone());

        Self {
            program_id,
            bench: bench_rc,
            governance: governance_bench,
            token_metadata: token_metadata_bench,
            merkle_tree: merkle_tree_bench,
        }
    }

    #[allow(dead_code)]
    pub async fn with_registrar(
        &mut self,
        realm_cookie: &RealmCookie
    ) -> Result<RegistrarCookie, BanksClientError> {
        self.with_registrar_using_ix(realm_cookie, NopOverride, None).await
    }

    #[allow(dead_code)]
    pub async fn with_registrar_using_ix<F: Fn(&mut Instruction)>(
        &mut self,
        realm_cookie: &RealmCookie,
        instruction_override: F,
        signers_override: Option<&[&Keypair]>
    ) -> Result<RegistrarCookie, BanksClientError> {
        let registrar_key = get_registrar_address(
            &realm_cookie.address,
            &realm_cookie.account.community_mint
        );

        let max_collections = 10;
        let data = anchor_lang::InstructionData::data(
            &(gpl_nft_voter::instruction::CreateRegistrar {
                max_collections,
            })
        );

        let accounts = anchor_lang::ToAccountMetas::to_account_metas(
            &(gpl_nft_voter::accounts::CreateRegistrar {
                registrar: registrar_key,
                realm: realm_cookie.address,
                governance_program_id: self.governance.program_id,
                governing_token_mint: realm_cookie.account.community_mint,
                realm_authority: realm_cookie.get_realm_authority().pubkey(),
                payer: self.bench.payer.pubkey(),
                system_program: solana_sdk::system_program::id(),
            }),
            None
        );

        let mut create_registrar_ix = Instruction {
            program_id: gpl_nft_voter::id(),
            accounts,
            data,
        };

        instruction_override(&mut create_registrar_ix);

        let default_signers = &[&realm_cookie.realm_authority];
        let signers = signers_override.unwrap_or(default_signers);

        self.bench.process_transaction(&[create_registrar_ix], Some(signers)).await?;

        let account = Registrar {
            governance_program_id: self.governance.program_id,
            realm: realm_cookie.address,
            governing_token_mint: realm_cookie.account.community_mint,
            collection_configs: vec![],
            reserved: [0; 128],
        };

        Ok(RegistrarCookie {
            address: registrar_key,
            account,
            realm_authority: realm_cookie.get_realm_authority(),
            max_collections,
        })
    }

    #[allow(dead_code)]
    pub async fn with_voter_weight_record(
        &self,
        registrar_cookie: &RegistrarCookie,
        voter_cookie: &WalletCookie
    ) -> Result<VoterWeightRecordCookie, BanksClientError> {
        self.with_voter_weight_record_using_ix(registrar_cookie, voter_cookie, NopOverride).await
    }

    #[allow(dead_code)]
    pub async fn with_voter_weight_record_using_ix<F: Fn(&mut Instruction)>(
        &self,
        registrar_cookie: &RegistrarCookie,
        voter_cookie: &WalletCookie,
        instruction_override: F
    ) -> Result<VoterWeightRecordCookie, BanksClientError> {
        let governing_token_owner = voter_cookie.address;

        let (voter_weight_record_key, _) = Pubkey::find_program_address(
            &[
                b"voter-weight-record".as_ref(),
                registrar_cookie.account.realm.as_ref(),
                registrar_cookie.account.governing_token_mint.as_ref(),
                governing_token_owner.as_ref(),
            ],
            &gpl_nft_voter::id()
        );

        let data = anchor_lang::InstructionData::data(
            &(gpl_nft_voter::instruction::CreateVoterWeightRecord {
                governing_token_owner,
            })
        );

        let accounts = gpl_nft_voter::accounts::CreateVoterWeightRecord {
            governance_program_id: self.governance.program_id,
            realm: registrar_cookie.account.realm,
            realm_governing_token_mint: registrar_cookie.account.governing_token_mint,
            voter_weight_record: voter_weight_record_key,
            payer: self.bench.payer.pubkey(),
            system_program: solana_sdk::system_program::id(),
        };

        let mut create_voter_weight_record_ix = Instruction {
            program_id: gpl_nft_voter::id(),
            accounts: anchor_lang::ToAccountMetas::to_account_metas(&accounts, None),
            data,
        };

        instruction_override(&mut create_voter_weight_record_ix);

        self.bench.process_transaction(&[create_voter_weight_record_ix], None).await?;

        let account = VoterWeightRecord {
            realm: registrar_cookie.account.realm,
            governing_token_mint: registrar_cookie.account.governing_token_mint,
            governing_token_owner,
            voter_weight: 0,
            voter_weight_expiry: Some(0),
            weight_action: None,
            weight_action_target: None,
            reserved: [0; 8],
        };

        Ok(VoterWeightRecordCookie {
            address: voter_weight_record_key,
            account,
        })
    }

    #[allow(dead_code)]
    pub async fn with_max_voter_weight_record(
        &mut self,
        registrar_cookie: &RegistrarCookie
    ) -> Result<MaxVoterWeightRecordCookie, BanksClientError> {
        self.with_max_voter_weight_record_using_ix(registrar_cookie, NopOverride).await
    }

    #[allow(dead_code)]
    pub async fn with_max_voter_weight_record_using_ix<F: Fn(&mut Instruction)>(
        &mut self,
        registrar_cookie: &RegistrarCookie,
        instruction_override: F
    ) -> Result<MaxVoterWeightRecordCookie, BanksClientError> {
        let max_voter_weight_record_key = get_max_voter_weight_record_address(
            &registrar_cookie.account.realm,
            &registrar_cookie.account.governing_token_mint
        );

        let data = anchor_lang::InstructionData::data(
            &(gpl_nft_voter::instruction::CreateMaxVoterWeightRecord {})
        );

        let accounts = gpl_nft_voter::accounts::CreateMaxVoterWeightRecord {
            governance_program_id: self.governance.program_id,
            realm: registrar_cookie.account.realm,
            realm_governing_token_mint: registrar_cookie.account.governing_token_mint,
            max_voter_weight_record: max_voter_weight_record_key,
            payer: self.bench.payer.pubkey(),
            system_program: solana_sdk::system_program::id(),
        };

        let mut create_max_voter_weight_record_ix = Instruction {
            program_id: gpl_nft_voter::id(),
            accounts: anchor_lang::ToAccountMetas::to_account_metas(&accounts, None),
            data,
        };

        instruction_override(&mut create_max_voter_weight_record_ix);

        self.bench.process_transaction(&[create_max_voter_weight_record_ix], None).await?;

        let account = MaxVoterWeightRecord {
            realm: registrar_cookie.account.realm,
            governing_token_mint: registrar_cookie.account.governing_token_mint,
            max_voter_weight: 0,
            max_voter_weight_expiry: Some(0),
            reserved: [0; 8],
        };

        Ok(MaxVoterWeightRecordCookie {
            account,
            address: max_voter_weight_record_key,
        })
    }

    #[allow(dead_code)]
    pub async fn update_voter_weight_record(
        &self,
        registrar_cookie: &RegistrarCookie,
        voter_weight_record_cookie: &mut VoterWeightRecordCookie,
        voter_weight_action: VoterWeightAction,
        nft_action_ticket_cookies: &[&NftVoteTicketCookie]
    ) -> Result<(), BanksClientError> {
        let data = anchor_lang::InstructionData::data(
            &(gpl_nft_voter::instruction::UpdateVoterWeightRecord {
                voter_weight_action,
            })
        );

        let accounts = gpl_nft_voter::accounts::UpdateVoterWeightRecord {
            registrar: registrar_cookie.address,
            voter_weight_record: voter_weight_record_cookie.address,
            payer: self.bench.payer.pubkey(),
        };

        let mut account_metas = anchor_lang::ToAccountMetas::to_account_metas(&accounts, None);

        for nft_action_ticket_cookie in nft_action_ticket_cookies {
            let nft_action_ticket = nft_action_ticket_cookie.address;
            account_metas.push(AccountMeta::new(nft_action_ticket, false));
        }

        let instructions = vec![Instruction {
            program_id: gpl_nft_voter::id(),
            accounts: account_metas,
            data,
        }];

        self.bench.process_transaction(&instructions, None).await
    }

    #[allow(dead_code)]
    pub async fn relinquish_nft_vote(
        &mut self,
        registrar_cookie: &RegistrarCookie,
        voter_weight_record_cookie: &VoterWeightRecordCookie,
        proposal_cookie: &ProposalCookie,
        voter_cookie: &WalletCookie,
        voter_token_owner_record_cookie: &TokenOwnerRecordCookie,
        nft_vote_record_cookies: &Vec<NftVoteRecordCookie>
    ) -> Result<(), BanksClientError> {
        let data = anchor_lang::InstructionData::data(
            &(gpl_nft_voter::instruction::RelinquishNftVote {})
        );

        let vote_record_key = vote_record::get_vote_record_address(
            &self.governance.program_id,
            &proposal_cookie.address,
            &voter_token_owner_record_cookie.address
        );

        let accounts = gpl_nft_voter::accounts::RelinquishNftVote {
            registrar: registrar_cookie.address,
            voter_weight_record: voter_weight_record_cookie.address,
            governance: proposal_cookie.account.governance,
            proposal: proposal_cookie.address,
            vote_record: vote_record_key,
            beneficiary: self.bench.payer.pubkey(),
            voter_token_owner_record: voter_token_owner_record_cookie.address,
            voter_authority: voter_cookie.address,
        };

        let mut account_metas = anchor_lang::ToAccountMetas::to_account_metas(&accounts, None);

        for nft_vote_record_cookie in nft_vote_record_cookies {
            account_metas.push(AccountMeta::new(nft_vote_record_cookie.address, false));
        }

        let relinquish_nft_vote_ix = Instruction {
            program_id: gpl_nft_voter::id(),
            accounts: account_metas,
            data,
        };

        self.bench.process_transaction(
            &[relinquish_nft_vote_ix],
            Some(&[&voter_cookie.signer])
        ).await?;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn with_collection(
        &mut self,
        registrar_cookie: &RegistrarCookie,
        nft_collection_cookie: &NftCollectionCookie,
        max_voter_weight_record_cookie: &MaxVoterWeightRecordCookie,
        args: Option<ConfigureCollectionArgs>
    ) -> Result<CollectionConfigCookie, BanksClientError> {
        self.with_collection_using_ix(
            registrar_cookie,
            nft_collection_cookie,
            max_voter_weight_record_cookie,
            args,
            NopOverride,
            None
        ).await
    }

    #[allow(dead_code)]
    pub async fn with_collection_using_ix<F: Fn(&mut Instruction)>(
        &mut self,
        registrar_cookie: &RegistrarCookie,
        nft_collection_cookie: &NftCollectionCookie,
        max_voter_weight_record_cookie: &MaxVoterWeightRecordCookie,
        args: Option<ConfigureCollectionArgs>,
        instruction_override: F,
        signers_override: Option<&[&Keypair]>
    ) -> Result<CollectionConfigCookie, BanksClientError> {
        let args = args.unwrap_or_default();

        let data = anchor_lang::InstructionData::data(
            &(gpl_nft_voter::instruction::ConfigureCollection {
                weight: args.weight,
                size: args.size,
            })
        );

        let accounts = gpl_nft_voter::accounts::ConfigureCollection {
            registrar: registrar_cookie.address,
            realm: registrar_cookie.account.realm,
            realm_authority: registrar_cookie.realm_authority.pubkey(),
            collection: nft_collection_cookie.mint,
            max_voter_weight_record: max_voter_weight_record_cookie.address,
        };

        let mut configure_collection_ix = Instruction {
            program_id: gpl_nft_voter::id(),
            accounts: anchor_lang::ToAccountMetas::to_account_metas(&accounts, None),
            data,
        };

        instruction_override(&mut configure_collection_ix);

        let default_signers = &[&registrar_cookie.realm_authority];
        let signers = signers_override.unwrap_or(default_signers);

        self.bench.process_transaction(&[configure_collection_ix], Some(signers)).await?;

        let collection_config = CollectionConfig {
            collection: nft_collection_cookie.mint,
            size: args.size,
            weight: args.weight,
            reserved: [0; 8],
        };

        Ok(CollectionConfigCookie { collection_config })
    }

    /// Casts NFT Vote and spl-gov Vote
    #[allow(dead_code)]
    pub async fn cast_nft_vote(
        &mut self,
        registrar_cookie: &RegistrarCookie,
        voter_weight_record_cookie: &VoterWeightRecordCookie,
        max_voter_weight_record_cookie: &MaxVoterWeightRecordCookie,
        proposal_cookie: &ProposalCookie,
        nft_voter_cookie: &WalletCookie,
        voter_token_owner_record_cookie: &TokenOwnerRecordCookie,
        nft_action_ticket_cookies: &[&NftVoteTicketCookie],
        args: Option<CastNftVoteArgs>
    ) -> Result<Vec<NftVoteRecordCookie>, BanksClientError> {
        let args = args.unwrap_or_default();
        let data = anchor_lang::InstructionData::data(
            &(gpl_nft_voter::instruction::CastNftVote {
                proposal: proposal_cookie.address,
            })
        );

        let accounts = gpl_nft_voter::accounts::CastNftVote {
            registrar: registrar_cookie.address,
            voter_weight_record: voter_weight_record_cookie.address,
            voter_token_owner_record: voter_token_owner_record_cookie.address,
            voter_authority: nft_voter_cookie.address,
            payer: self.bench.payer.pubkey(),
            system_program: solana_sdk::system_program::id(),
        };

        let mut account_metas = anchor_lang::ToAccountMetas::to_account_metas(&accounts, None);
        let mut nft_vote_record_cookies = vec![];

        for nft_action_ticket_cookie in nft_action_ticket_cookies {
            let nft_mint = &nft_action_ticket_cookie.nft_mint;

            let nft_action_ticket = nft_action_ticket_cookie.address;
            let nft_action_ticket_info = AccountMeta::new(nft_action_ticket, false);

            let nft_vote_record = get_nft_vote_record_address(&proposal_cookie.address, &nft_mint);
            let nft_vote_record_info = AccountMeta::new(nft_vote_record, false);

            account_metas.push(nft_action_ticket_info);
            account_metas.push(nft_vote_record_info);

            let account = NftVoteRecord {
                proposal: proposal_cookie.address,
                nft_mint: nft_mint.clone(),
                governing_token_owner: voter_weight_record_cookie.account.governing_token_owner,
                account_discriminator: NftVoteRecord::ACCOUNT_DISCRIMINATOR,
                reserved: [0; 8],
            };

            nft_vote_record_cookies.push(NftVoteRecordCookie {
                address: nft_vote_record,
                account,
            });
        }

        let cast_nft_vote_ix = Instruction {
            program_id: gpl_nft_voter::id(),
            accounts: account_metas,
            data,
        };

        let mut instruction = vec![cast_nft_vote_ix];

        if args.cast_spl_gov_vote {
            // spl-gov cast vote
            let vote = Vote::Approve(
                vec![VoteChoice {
                    rank: 0,
                    weight_percentage: 100,
                }]
            );

            let cast_vote_ix = cast_vote(
                &self.governance.program_id,
                &registrar_cookie.account.realm,
                &proposal_cookie.account.governance,
                &proposal_cookie.address,
                &proposal_cookie.account.token_owner_record,
                &voter_token_owner_record_cookie.address,
                &nft_voter_cookie.address,
                &proposal_cookie.account.governing_token_mint,
                &self.bench.payer.pubkey(),
                Some(voter_weight_record_cookie.address),
                Some(max_voter_weight_record_cookie.address),
                vote
            );

            instruction.push(cast_vote_ix);
        }

        self.bench.process_transaction(&instruction, Some(&[&nft_voter_cookie.signer])).await?;

        Ok(nft_vote_record_cookies)
    }

    #[allow(dead_code)]
    pub async fn with_create_nft_action_ticket(
        &mut self,
        registrar_cookie: &RegistrarCookie,
        voter_weight_record_cookie: &VoterWeightRecordCookie,
        voter_cookie: &WalletCookie,
        nft_cookies: &[&NftCookie],
        action: &VoterWeightAction
    ) -> Result<Vec<NftVoteTicketCookie>, BanksClientError> {
        self.with_create_nft_action_ticket_using_ix(
            registrar_cookie,
            voter_weight_record_cookie,
            voter_cookie,
            nft_cookies,
            action,
            NopOverride,
            None
        ).await
    }

    #[allow(dead_code)]
    pub async fn with_create_nft_action_ticket_using_ix<F: Fn(&mut Instruction)>(
        &mut self,
        registrar_cookie: &RegistrarCookie,
        voter_weight_record_cookie: &VoterWeightRecordCookie,
        voter_cookie: &WalletCookie,
        nft_cookies: &[&NftCookie],
        action: &VoterWeightAction,
        instruction_override: F,
        signers_override: Option<&[&Keypair]>
    ) -> Result<Vec<NftVoteTicketCookie>, BanksClientError> {
        let accounts = gpl_nft_voter::accounts::CreateNftActionTicket {
            registrar: registrar_cookie.address,
            voter_weight_record: voter_weight_record_cookie.address,
            voter_authority: voter_cookie.address,
            payer: self.bench.payer.pubkey(),
            system_program: solana_sdk::system_program::id(),
        };

        let data = anchor_lang::InstructionData::data(
            &(gpl_nft_voter::instruction::CreateNftActionTicket {
                voter_weight_action: action.clone(),
            })
        );

        let mut verify_nft_info_ix = Instruction {
            program_id: gpl_nft_voter::id(),
            accounts: anchor_lang::ToAccountMetas::to_account_metas(&accounts, None),
            data,
        };

        let mut nft_action_ticket_cookies = vec![];
        let ticket_type = format!("nft-{}-ticket", &action).to_string();
        for nft_cookie in nft_cookies {
            let nft_action_ticket = get_nft_action_ticket_address(
                &ticket_type,
                &registrar_cookie.address,
                &voter_cookie.address,
                &nft_cookie.mint_cookie.address
            ).0;

            verify_nft_info_ix.accounts.push(AccountMeta::new_readonly(nft_cookie.address, false));
            verify_nft_info_ix.accounts.push(AccountMeta::new_readonly(nft_cookie.metadata, false));
            verify_nft_info_ix.accounts.push(AccountMeta::new(nft_action_ticket, false));

            nft_action_ticket_cookies.push(NftVoteTicketCookie {
                nft_mint: nft_cookie.mint_cookie.address.clone(),
                address: nft_action_ticket.clone(),
            });
        }

        instruction_override(&mut verify_nft_info_ix);
        let default_signers = &[&voter_cookie.signer];
        let signers = signers_override.unwrap_or(default_signers);

        self.bench.process_transaction(&[verify_nft_info_ix], Some(signers)).await?;

        Ok(nft_action_ticket_cookies)
    }

    #[allow(dead_code)]
    pub async fn with_create_cnft_action_ticket(
        &mut self,
        registrar_cookie: &RegistrarCookie,
        voter_weight_record_cookie: &VoterWeightRecordCookie,
        voter_cookie: &WalletCookie,
        leaf_cookies: &[&LeafArgs],
        leaf_verification_cookies: &[&LeafVerificationCookie],
        proofs: &[&Vec<AccountMeta>],
        action: &VoterWeightAction
    ) -> Result<Vec<NftVoteTicketCookie>, BanksClientError> {
        self.with_create_cnft_action_ticket_using_ix(
            registrar_cookie,
            voter_weight_record_cookie,
            voter_cookie,
            leaf_cookies,
            leaf_verification_cookies,
            &proofs,
            &action,
            NopOverride,
            None
        ).await
    }

    #[allow(dead_code)]
    pub async fn with_create_cnft_action_ticket_using_ix<F: Fn(&mut Instruction)>(
        &mut self,
        registrar_cookie: &RegistrarCookie,
        voter_weight_record_cookie: &VoterWeightRecordCookie,
        voter_cookie: &WalletCookie,
        leaf_cookies: &[&LeafArgs],
        leaf_verification_cookies: &[&LeafVerificationCookie],
        proofs: &[&Vec<AccountMeta>],
        action: &VoterWeightAction,
        instruction_override: F,
        signers_override: Option<&[&Keypair]>
    ) -> Result<Vec<NftVoteTicketCookie>, BanksClientError> {
        let params: Vec<LeafVerificationCookie> = leaf_verification_cookies
            .to_vec()
            .into_iter()
            .map(|v| v.clone())
            .collect();

        let accounts = gpl_nft_voter::accounts::CreateCnftActionTicket {
            registrar: registrar_cookie.address,
            voter_weight_record: voter_weight_record_cookie.address,
            voter_authority: voter_cookie.address,
            payer: self.bench.payer.pubkey(),
            compression_program: spl_account_compression::id(),
            system_program: solana_sdk::system_program::id(),
        };

        let data = anchor_lang::InstructionData::data(
            &(gpl_nft_voter::instruction::CreateCnftActionTicket {
                voter_weight_action: action.clone(),
                params,
            })
        );

        let mut verify_cnft_info_ix = Instruction {
            program_id: gpl_nft_voter::id(),
            accounts: anchor_lang::ToAccountMetas::to_account_metas(&accounts, None),
            data,
        };

        let mut nft_action_ticket_cookies = vec![];
        let ticket_type = format!("nft-{}-ticket", &action).to_string();
        for i in 0..leaf_verification_cookies.len() {
            let tree_address = leaf_cookies[i].tree_address;
            let tree_account_info = AccountMeta::new_readonly(tree_address, false);
            let proof = &mut proofs[i].clone();
            let asset_id = &leaf_cookies[i].asset_id;

            let cnft_action_ticket = get_nft_action_ticket_address(
                &ticket_type,
                &registrar_cookie.address,
                &voter_cookie.address,
                asset_id
            ).0;
            let cnft_action_ticket_info = AccountMeta::new(cnft_action_ticket, false);

            verify_cnft_info_ix.accounts.push(tree_account_info);
            verify_cnft_info_ix.accounts.append(proof);
            verify_cnft_info_ix.accounts.push(cnft_action_ticket_info);

            nft_action_ticket_cookies.push(NftVoteTicketCookie {
                nft_mint: asset_id.clone(),
                address: cnft_action_ticket.clone(),
            });
        }

        instruction_override(&mut verify_cnft_info_ix);
        let default_signers = &[&voter_cookie.signer];
        let signers = signers_override.unwrap_or(default_signers);

        self.bench.process_transaction(&[verify_cnft_info_ix], Some(signers)).await?;

        Ok(nft_action_ticket_cookies)
    }

    #[allow(dead_code)]
    pub async fn get_registrar_account(&mut self, registrar: &Pubkey) -> Registrar {
        self.bench.get_anchor_account::<Registrar>(*registrar).await
    }

    #[allow(dead_code)]
    pub async fn get_nft_vote_record_account(&mut self, nft_vote_record: &Pubkey) -> NftVoteRecord {
        self.bench.get_borsh_account::<NftVoteRecord>(nft_vote_record).await
    }

    #[allow(dead_code)]
    pub async fn get_max_voter_weight_record(
        &self,
        max_voter_weight_record: &Pubkey
    ) -> MaxVoterWeightRecord {
        self.bench.get_anchor_account(*max_voter_weight_record).await
    }

    #[allow(dead_code)]
    pub async fn get_voter_weight_record(&self, voter_weight_record: &Pubkey) -> VoterWeightRecord {
        self.bench.get_anchor_account(*voter_weight_record).await
    }

    #[allow(dead_code)]
    pub async fn get_nft_action_ticket(&mut self, cnft_action_ticket: &Pubkey) -> NftActionTicket {
        self.bench.get_borsh_account::<NftActionTicket>(cnft_action_ticket).await
    }
}
