use super::{list_branch_node::ListBranchNode, node::NodeInternal, split_branch_node::SplitBranchNode};

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum BranchNode {
    ListBranch(ListBranchNode),
    SplitBranch(SplitBranchNode),
}

impl From<BranchNode> for NodeInternal {
    fn from(node: BranchNode) -> Self {
        match node {
            BranchNode::ListBranch(n) => NodeInternal::ListBranch(n),
            BranchNode::SplitBranch(n) => NodeInternal::SplitBranch(n),
        }
    }
}
