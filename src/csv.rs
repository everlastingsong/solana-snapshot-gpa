use crate::filter::AccountFilter;

use serde::Serialize;
use solana_snapshot_etl::append_vec::{AppendVec, StoredAccountMeta};
use solana_snapshot_etl::append_vec_iter;
use std::io::Stdout;
use std::rc::Rc;
use base64;

pub(crate) struct CsvDumper {
    writer: csv::Writer<Stdout>,
    accounts_count: u64,
    filter: AccountFilter,
}

#[derive(Serialize)]
struct Record {
    pubkey: String,
    owner: String,
    data_len: u64,
    lamports: u64,
    slot: u64,
    id: u64,
    offset: usize,
    write_version: u64,
    data: String,
}

impl CsvDumper {
    pub(crate) fn new(filter: AccountFilter, noheader: bool) -> Self {
        let writer = csv::WriterBuilder::new()
            .has_headers(!noheader)
            .from_writer(std::io::stdout());
        
        Self {
            writer,
            accounts_count: 0,
            filter,
        }
    }

    pub(crate) fn dump_append_vec(&mut self, slot: u64, id: u64, append_vec: AppendVec) {
        for account in append_vec_iter(Rc::new(append_vec)) {
            let account = account.access().unwrap();
            if self.filter.is_match(&account) {
                self.dump_account(slot, id, account);
            }
        }
    }

    pub(crate) fn dump_account(&mut self, slot: u64, id: u64, account: StoredAccountMeta) {
        let record = Record {
            pubkey: account.meta.pubkey.to_string(),
            owner: account.account_meta.owner.to_string(),
            data_len: account.meta.data_len,
            lamports: account.account_meta.lamports,
            slot,
            id,
            offset: account.offset,
            write_version: account.meta.write_version,
            data: base64::encode(account.data),
        };
        if self.writer.serialize(record).is_err() {
            std::process::exit(1); // if stdout closes, silently exit
        }
        self.accounts_count += 1;
    }
}

impl Drop for CsvDumper {
    fn drop(&mut self) {
    }
}
