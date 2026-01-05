use bees::Record;

#[derive(Record)]
#[record(url = "hello")]
// #[record_name("thing")]
pub struct MyRecord;