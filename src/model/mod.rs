pub mod blocks;
pub mod consts;
pub mod effect;
mod game_model;
pub mod types;
pub use game_model::*;


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessing_block_in_region() {
        let reg = blocks::Region::new();
        let chunk = reg.get_chunk([1, 2, 3]);
        let block = chunk.get_block([1, 2, 3]);

        assert_eq!(block.kind, blocks::BlockKind::Air);
    }
}
