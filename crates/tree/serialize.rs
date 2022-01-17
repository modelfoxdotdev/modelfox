use bitvec::prelude::*;
use ndarray::prelude::*;
use num::ToPrimitive;

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct Regressor {
	#[buffalo(id = 0, required)]
	pub bias: f32,
	#[buffalo(id = 1, required)]
	pub trees: Vec<Tree>,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct BinaryClassifier {
	#[buffalo(id = 0, required)]
	pub bias: f32,
	#[buffalo(id = 1, required)]
	pub trees: Vec<Tree>,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct MulticlassClassifier {
	#[buffalo(id = 0, required)]
	pub biases: Array1<f32>,
	#[buffalo(id = 1, required)]
	pub trees: Array2<Tree>,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct Tree {
	#[buffalo(id = 0, required)]
	pub nodes: Vec<Node>,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "static", value_size = 8)]
pub enum Node {
	#[buffalo(id = 0)]
	Branch(BranchNode),
	#[buffalo(id = 1)]
	Leaf(LeafNode),
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct BranchNode {
	#[buffalo(id = 0, required)]
	pub left_child_index: u64,
	#[buffalo(id = 1, required)]
	pub right_child_index: u64,
	#[buffalo(id = 2, required)]
	pub split: BranchSplit,
	#[buffalo(id = 3, required)]
	pub examples_fraction: f32,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "static", value_size = 8)]
pub enum BranchSplit {
	#[buffalo(id = 0)]
	Continuous(BranchSplitContinuous),
	#[buffalo(id = 1)]
	Discrete(BranchSplitDiscrete),
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct BranchSplitContinuous {
	#[buffalo(id = 0, required)]
	pub feature_index: u64,
	#[buffalo(id = 1, required)]
	pub split_value: f32,
	#[buffalo(id = 2, required)]
	pub invalid_values_direction: SplitDirection,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct BranchSplitDiscrete {
	#[buffalo(id = 0, required)]
	pub feature_index: u64,
	#[buffalo(id = 1, required)]
	pub directions: BitVec<u8, Lsb0>,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "static", value_size = 0)]
pub enum SplitDirection {
	#[buffalo(id = 0)]
	Left,
	#[buffalo(id = 1)]
	Right,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct LeafNode {
	#[buffalo(id = 0, required)]
	pub value: f64,
	#[buffalo(id = 1, required)]
	pub examples_fraction: f32,
}

pub(crate) fn serialize_regressor(
	regressor: &crate::Regressor,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<RegressorWriter> {
	let trees = regressor
		.trees
		.iter()
		.map(|tree| {
			let tree = serialize_tree(tree, writer);
			writer.write(&tree)
		})
		.collect::<Vec<_>>();
	let trees = writer.write(&trees);
	writer.write(&RegressorWriter {
		bias: regressor.bias,
		trees,
	})
}

pub(crate) fn serialize_binary_classifier(
	binary_classifier: &crate::BinaryClassifier,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<BinaryClassifierWriter> {
	let trees = binary_classifier
		.trees
		.iter()
		.map(|tree| {
			let tree = serialize_tree(tree, writer);
			writer.write(&tree)
		})
		.collect::<Vec<_>>();
	let trees = writer.write(&trees);
	writer.write(&BinaryClassifierWriter {
		bias: binary_classifier.bias,
		trees,
	})
}

pub(crate) fn serialize_multiclass_classifier(
	multiclass_classifier: &crate::MulticlassClassifier,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<MulticlassClassifierWriter> {
	let biases = writer.write(&multiclass_classifier.biases);
	let trees = multiclass_classifier.trees.map(|tree| {
		let tree = serialize_tree(tree, writer);
		writer.write(&tree)
	});
	let trees = writer.write(&trees);
	writer.write(&MulticlassClassifierWriter { biases, trees })
}

fn serialize_tree(tree: &crate::Tree, writer: &mut buffalo::Writer) -> TreeWriter {
	let nodes = tree
		.nodes
		.iter()
		.map(|node| serialize_node(node, writer))
		.collect::<Vec<_>>();
	let nodes = writer.write(&nodes);
	TreeWriter { nodes }
}

fn serialize_node(node: &crate::Node, writer: &mut buffalo::Writer) -> NodeWriter {
	match node {
		crate::Node::Branch(node) => {
			let split = serialize_branch_split(&node.split, writer);
			let node = writer.write(&BranchNodeWriter {
				left_child_index: node.left_child_index.to_u64().unwrap(),
				right_child_index: node.right_child_index.to_u64().unwrap(),
				split,
				examples_fraction: node.examples_fraction,
			});
			NodeWriter::Branch(node)
		}
		crate::Node::Leaf(node) => {
			let node = writer.write(&LeafNodeWriter {
				value: node.value,
				examples_fraction: node.examples_fraction,
			});
			NodeWriter::Leaf(node)
		}
	}
}

fn serialize_branch_split(
	branch_split: &crate::BranchSplit,
	writer: &mut buffalo::Writer,
) -> BranchSplitWriter {
	match branch_split {
		crate::BranchSplit::Continuous(split) => {
			let invalid_values_direction =
				serialize_split_direction(&split.invalid_values_direction, writer);
			let split = writer.write(&BranchSplitContinuousWriter {
				feature_index: split.feature_index.to_u64().unwrap(),
				split_value: split.split_value,
				invalid_values_direction,
			});
			BranchSplitWriter::Continuous(split)
		}
		crate::BranchSplit::Discrete(split) => {
			let directions = writer.write(&split.directions);
			let split = writer.write(&BranchSplitDiscreteWriter {
				feature_index: split.feature_index.to_u64().unwrap(),
				directions,
			});
			BranchSplitWriter::Discrete(split)
		}
	}
}

fn serialize_split_direction(
	split_direction: &crate::SplitDirection,
	_writer: &mut buffalo::Writer,
) -> SplitDirectionWriter {
	match split_direction {
		crate::SplitDirection::Left => SplitDirectionWriter::Left,
		crate::SplitDirection::Right => SplitDirectionWriter::Right,
	}
}

pub(crate) fn deserialize_regressor(model: RegressorReader) -> crate::Regressor {
	let bias = model.bias();
	let trees = model
		.trees()
		.iter()
		.map(deserialize_tree)
		.collect::<Vec<_>>();
	crate::Regressor { bias, trees }
}

pub(crate) fn deserialize_binary_classifier(
	model: BinaryClassifierReader,
) -> crate::BinaryClassifier {
	let bias = model.bias();
	let trees = model
		.trees()
		.iter()
		.map(deserialize_tree)
		.collect::<Vec<_>>();
	crate::BinaryClassifier { bias, trees }
}

pub(crate) fn deserialize_multiclass_classifier(
	model: MulticlassClassifierReader,
) -> crate::MulticlassClassifier {
	let biases = model.biases();
	let trees = model.trees().mapv(deserialize_tree);
	crate::MulticlassClassifier { biases, trees }
}

fn deserialize_tree(tree: TreeReader) -> crate::Tree {
	let nodes = tree
		.nodes()
		.iter()
		.map(deserialize_node)
		.collect::<Vec<_>>();
	crate::Tree { nodes }
}

fn deserialize_node(node: NodeReader) -> crate::Node {
	match node {
		NodeReader::Branch(node) => {
			let node = node.read();
			let left_child_index = node.left_child_index().to_usize().unwrap();
			let right_child_index = node.right_child_index().to_usize().unwrap();
			let examples_fraction = node.examples_fraction();
			let split = deserialize_branch_split(node.split());
			crate::Node::Branch(crate::BranchNode {
				left_child_index,
				right_child_index,
				split,
				examples_fraction,
			})
		}
		NodeReader::Leaf(node) => {
			let node = node.read();
			let value = node.value();
			let examples_fraction = node.examples_fraction();
			crate::Node::Leaf(crate::LeafNode {
				value,
				examples_fraction,
			})
		}
	}
}

fn deserialize_branch_split(branch_split: BranchSplitReader) -> crate::BranchSplit {
	match branch_split {
		BranchSplitReader::Continuous(split) => {
			let split = split.read();
			let feature_index = split.feature_index().to_usize().unwrap();
			let split_value = split.split_value();
			let invalid_values_direction = match split.invalid_values_direction() {
				SplitDirectionReader::Left(_) => crate::SplitDirection::Left,
				SplitDirectionReader::Right(_) => crate::SplitDirection::Right,
			};
			crate::BranchSplit::Continuous(crate::BranchSplitContinuous {
				feature_index,
				split_value,
				invalid_values_direction,
			})
		}
		BranchSplitReader::Discrete(split) => {
			let split = split.read();
			let feature_index = split.feature_index().to_usize().unwrap();
			let directions = split.directions().to_owned();
			crate::BranchSplit::Discrete(crate::BranchSplitDiscrete {
				feature_index,
				directions,
			})
		}
	}
}
