fn main() {
	let stdin = std::io::stdin();
	let stdin = stdin.lock();
	let stdin = std::io::BufReader::new(stdin);
	let action_iter = term_parser::ActionIter::new(stdin);
	for action in action_iter {
		println!("{:?}", action);
	}
}
