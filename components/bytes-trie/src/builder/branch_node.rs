use super::{list_branch_node::ListBranchNode, node::Node, split_branch_node::SplitBranchNode};

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum BranchNode {
    ListBranch(ListBranchNode),
    SplitBranch(SplitBranchNode),
}

impl From<BranchNode> for Node {
    fn from(node: BranchNode) -> Self {
        match node {
            BranchNode::ListBranch(n) => Node::ListBranch(n),
            BranchNode::SplitBranch(n) => Node::SplitBranch(n),
        }
    }
}
