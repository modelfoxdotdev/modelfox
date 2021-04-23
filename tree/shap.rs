use crate::{
	BranchNode, BranchSplit, BranchSplitContinuous, BranchSplitDiscrete, Node, SplitDirection, Tree,
};
use ndarray::prelude::*;
use num::ToPrimitive;

pub struct ComputeShapValuesForExampleOutput {
	pub baseline_value: f32,
	pub output_value: f32,
	pub feature_contribution_values: Vec<f32>,
}

/// Compute the SHAP values for a single class for a single example.
pub fn compute_shap_values_for_example(
	example: &[tangram_table::TableValue],
	trees: ArrayView1<Tree>,
	bias: f32,
) -> ComputeShapValuesForExampleOutput {
	let mut baseline_value = bias as f64;
	for tree in trees {
		baseline_value += compute_expectation(tree, 0);
	}
	let mut feature_contribution_values = vec![0.0; example.len()];
	for tree in trees {
		tree_shap(example, tree, feature_contribution_values.as_mut_slice());
	}
	let output_value = baseline_value + feature_contribution_values.iter().sum::<f64>();
	ComputeShapValuesForExampleOutput {
		baseline_value: baseline_value.to_f32().unwrap(),
		output_value: output_value.to_f32().unwrap(),
		feature_contribution_values: feature_contribution_values
			.iter()
			.map(|f| f.to_f32().unwrap())
			.collect(),
	}
}

/// This function, and the helper functions below it, are a direct port from https://github.com/slundberg/shap.
fn tree_shap(example: &[tangram_table::TableValue], tree: &Tree, phi: &mut [f64]) {
	let max_depth = max_depth(tree, 0, 0) + 2;
	let mut unique_path = vec![PathItem::new(); max_depth * (max_depth + 1) / 2];
	tree_shap_recursive(TreeShapRecursiveOptions {
		phi,
		example,
		tree,
		node_index: 0,
		unique_path: unique_path.as_mut_slice(),
		unique_depth: 0,
		parent_zero_fraction: 1.0,
		parent_one_fraction: 1.0,
		parent_feature_index: None,
	});
}

#[derive(Debug, Clone)]
struct PathItem {
	feature_index: Option<usize>,
	zero_fraction: f64,
	one_fraction: f64,
	pweight: f64,
}

impl PathItem {
	fn new() -> PathItem {
		PathItem {
			feature_index: None,
			zero_fraction: 0.0,
			one_fraction: 0.0,
			pweight: 0.0,
		}
	}
}

struct TreeShapRecursiveOptions<'a> {
	example: &'a [tangram_table::TableValue<'a>],
	node_index: usize,
	parent_feature_index: Option<usize>,
	parent_one_fraction: f64,
	parent_zero_fraction: f64,
	phi: &'a mut [f64],
	tree: &'a Tree,
	unique_depth: usize,
	unique_path: &'a mut [PathItem],
}

fn tree_shap_recursive(options: TreeShapRecursiveOptions) {
	let TreeShapRecursiveOptions {
		example,
		node_index,
		parent_feature_index,
		parent_one_fraction,
		parent_zero_fraction,
		phi,
		tree,
		unique_depth,
		unique_path,
	} = options;
	extend_path(ExtendPathOptions {
		unique_path,
		unique_depth,
		zero_fraction: parent_zero_fraction,
		one_fraction: parent_one_fraction,
		feature_index: parent_feature_index,
	});
	let mut unique_depth = unique_depth;
	let node = &tree.nodes[node_index];
	match node {
		Node::Leaf(node) => {
			for path_index in 1..=unique_depth {
				let weight = unwound_path_sum(unique_path, unique_depth, path_index);
				let path_item = &unique_path[path_index];
				let scale = weight * (path_item.one_fraction - path_item.zero_fraction);
				phi[path_item.feature_index.unwrap()] += scale * node.value as f64;
			}
		}
		Node::Branch(node) => {
			let (hot_child_index, cold_child_index) = compute_hot_cold_child(node, example);
			let hot_zero_fraction = tree.nodes[hot_child_index].examples_fraction() as f64
				/ node.examples_fraction as f64;
			let cold_zero_fraction = tree.nodes[cold_child_index].examples_fraction() as f64
				/ node.examples_fraction as f64;
			let mut incoming_zero_fraction = 1.0;
			let mut incoming_one_fraction = 1.0;
			let current_feature_index = node.split.feature_index();
			if let Some(path_index) = (1..=unique_depth)
				.find(|i| unique_path[*i].feature_index.unwrap() == current_feature_index)
			{
				incoming_zero_fraction = unique_path[path_index].zero_fraction;
				incoming_one_fraction = unique_path[path_index].one_fraction;
				unwind_path(unique_path, unique_depth, path_index);
				unique_depth -= 1;
			};
			let feature_index = node.split.feature_index();
			let (parent_path, child_path) = unique_path.split_at_mut(unique_depth + 1);
			child_path[0..parent_path.len()].clone_from_slice(parent_path);
			tree_shap_recursive(TreeShapRecursiveOptions {
				phi,
				example,
				tree,
				node_index: hot_child_index,
				unique_path: child_path,
				unique_depth: unique_depth + 1,
				parent_zero_fraction: hot_zero_fraction * incoming_zero_fraction,
				parent_one_fraction: incoming_one_fraction,
				parent_feature_index: Some(feature_index),
			});
			child_path[0..parent_path.len()].clone_from_slice(parent_path);
			tree_shap_recursive(TreeShapRecursiveOptions {
				phi,
				example,
				tree,
				node_index: cold_child_index,
				unique_path: child_path,
				unique_depth: unique_depth + 1,
				parent_zero_fraction: cold_zero_fraction * incoming_zero_fraction,
				parent_one_fraction: 0.0,
				parent_feature_index: Some(feature_index),
			});
		}
	};
}

struct ExtendPathOptions<'a> {
	unique_path: &'a mut [PathItem],
	unique_depth: usize,
	zero_fraction: f64,
	one_fraction: f64,
	feature_index: Option<usize>,
}

fn extend_path(options: ExtendPathOptions) {
	let ExtendPathOptions {
		feature_index,
		one_fraction,
		unique_depth,
		unique_path,
		zero_fraction,
	} = options;
	unique_path[unique_depth] = PathItem {
		feature_index,
		zero_fraction,
		one_fraction,
		pweight: if unique_depth == 0 { 1.0 } else { 0.0 },
	};
	if unique_depth == 0 {
		return;
	}
	for i in (0..unique_depth).rev() {
		unique_path[i + 1].pweight +=
			one_fraction * unique_path[i].pweight * (i + 1).to_f64().unwrap()
				/ (unique_depth + 1).to_f64().unwrap();
		unique_path[i].pweight =
			zero_fraction * unique_path[i].pweight * (unique_depth - i).to_f64().unwrap()
				/ (unique_depth + 1).to_f64().unwrap();
	}
}

fn unwind_path(unique_path: &mut [PathItem], unique_depth: usize, path_index: usize) {
	let one_fraction = unique_path[path_index].one_fraction;
	let zero_fraction = unique_path[path_index].zero_fraction;
	let mut next_one_portion = unique_path[unique_depth].pweight;
	for i in (0..unique_depth).rev() {
		if one_fraction != 0.0 {
			let tmp = unique_path[i].pweight;
			unique_path[i].pweight = next_one_portion * (unique_depth + 1).to_f64().unwrap()
				/ ((i + 1).to_f64().unwrap() * one_fraction);
			next_one_portion = tmp
				- unique_path[i].pweight * zero_fraction * (unique_depth - i).to_f64().unwrap()
					/ (unique_depth + 1).to_f64().unwrap();
		} else {
			unique_path[i].pweight = unique_path[i].pweight * (unique_depth + 1).to_f64().unwrap()
				/ (zero_fraction * (unique_depth - i).to_f64().unwrap());
		}
	}
	for i in path_index..unique_depth {
		unique_path[i].feature_index = unique_path[i + 1].feature_index;
		unique_path[i].zero_fraction = unique_path[i + 1].zero_fraction;
		unique_path[i].one_fraction = unique_path[i + 1].one_fraction;
	}
}

fn unwound_path_sum(unique_path: &[PathItem], unique_depth: usize, path_index: usize) -> f64 {
	let one_fraction = unique_path[path_index].one_fraction;
	let zero_fraction = unique_path[path_index].zero_fraction;
	let mut next_one_portion = unique_path[unique_depth].pweight;
	let mut total = 0.0;
	if one_fraction != 0.0 {
		for i in (0..unique_depth).rev() {
			let tmp = next_one_portion / ((i + 1).to_f64().unwrap() * one_fraction);
			total += tmp;
			next_one_portion =
				unique_path[i].pweight - tmp * zero_fraction * (unique_depth - i).to_f64().unwrap();
		}
	} else {
		for i in (0..unique_depth).rev() {
			total +=
				unique_path[i].pweight / (zero_fraction * (unique_depth - i).to_f64().unwrap());
		}
	}
	total * (unique_depth + 1).to_f64().unwrap()
}

fn compute_hot_cold_child(
	node: &BranchNode,
	example: &[tangram_table::TableValue],
) -> (usize, usize) {
	match &node.split {
		BranchSplit::Continuous(BranchSplitContinuous {
			feature_index,
			split_value,
			invalid_values_direction,
		}) => match example[*feature_index] {
			tangram_table::TableValue::Number(value) => {
				if value.is_nan() {
					if let SplitDirection::Left = invalid_values_direction {
						(node.left_child_index, node.right_child_index)
					} else {
						(node.right_child_index, node.left_child_index)
					}
				} else if value <= *split_value {
					(node.left_child_index, node.right_child_index)
				} else {
					(node.right_child_index, node.left_child_index)
				}
			}
			_ => unreachable!(),
		},
		BranchSplit::Discrete(BranchSplitDiscrete {
			feature_index,
			directions,
		}) => match example[*feature_index] {
			tangram_table::TableValue::Enum(value) => {
				let bin_index = value.map(|value| value.get()).unwrap_or(0);
				match (*directions.get(bin_index).unwrap()).into() {
					SplitDirection::Left => (node.left_child_index, node.right_child_index),
					SplitDirection::Right => (node.right_child_index, node.left_child_index),
				}
			}
			_ => unreachable!(),
		},
	}
}

fn max_depth(tree: &Tree, node_index: usize, depth: usize) -> usize {
	let current_node = &tree.nodes[node_index];
	if let Node::Leaf(_) = current_node {
		return depth;
	}
	let current_node = current_node.as_branch().unwrap();
	let left_child_index = current_node.left_child_index;
	let right_child_index = current_node.right_child_index;
	let left_depth = max_depth(tree, left_child_index, depth + 1);
	let right_depth = max_depth(tree, right_child_index, depth + 1);
	usize::max(left_depth, right_depth) + 1
}

fn compute_expectation(tree: &Tree, node_index: usize) -> f64 {
	let current_node = &tree.nodes[node_index];
	if let Node::Leaf(n) = current_node {
		return n.value;
	}
	let current_node = current_node.as_branch().unwrap();
	let left_child_index = current_node.left_child_index;
	let right_child_index = current_node.right_child_index;
	let left_child = &tree.nodes[left_child_index];
	let right_child = &tree.nodes[right_child_index];
	let left_value = compute_expectation(tree, left_child_index);
	let right_value = compute_expectation(tree, right_child_index);
	(left_child.examples_fraction() as f64 / current_node.examples_fraction as f64) * left_value
		+ (right_child.examples_fraction() as f64 / current_node.examples_fraction as f64)
			* right_value
}
