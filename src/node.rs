use super::Config;

#[derive(Copy, Clone, Debug)]
pub(crate) enum NodeType {
    INPUT,
    HIDDEN,
    OUTPUT,
}

#[derive(Copy, Clone)]
pub struct Node {
    pub(crate) idx: usize, // unique across all nodes (TODO: ... although, there's some consideration to be made for the fact that the inner neurons don't necessarily need to be a particular inner nueron, possibly we can "shift" the inner neurons as a mutation? Or we can manually shift the index for them such that various chromosomes don't overlap functions with each other's inner neurons. Not sure that's what we'd want, but it could be interesting to consider this "independence" of the inner neurons, by adjusting their assigned "idx" values algorithmically to "make sense" for a different model/constrant/design.)
    pub(crate) node_type: NodeType,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx
    }
}

impl Node {
    // fn idx(&self) -> usize { self.idx as usize }
    fn is_hidden(&self) -> bool { matches!(self.node_type, NodeType::HIDDEN) }
    fn is_input(&self) -> bool { matches!(self.node_type, NodeType::INPUT) }
    fn is_output(&self) -> bool { matches!(self.node_type, NodeType::OUTPUT) }
    pub fn get_name(&self) -> String {
		Config::get().node_name( self )
    }
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str( format!("{}", self.get_name()).as_str() )
        // f.debug_tuple( "" ).field( &self.idx ).field( &self.node_type ).finish()
    }
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_str = match self.node_type {
            NodeType::INPUT => "I",
            NodeType::HIDDEN => "H", 
            NodeType::OUTPUT => "O", 
        };
        write!(f, "{}{}", node_str, self.idx)
    }
}

