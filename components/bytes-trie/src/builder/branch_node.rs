use super::{
    list_branch_node::ListBranchNode, node::NodeInternal, split_branch_node::SplitBranchNode,
};

#[derive(Debug, Eq, PartialEq, Hash)]
pub(crate) enum BranchNode {
    ListBranch(ListBranchNode),
    SplitBranch(SplitBranchNode),
}

impl From<BranchNode> for NodeInternal {
    fn from(node: BranchNode) -> Self {
        match node {
            BranchNode::ListBranch(n) => NodeContent::ListBranch(n).into(),
            BranchNode::SplitBranch(n) => NodeContent::SplitBranch(n).into(),
        }
    }
}
