use std::{borrow::Cow, collections::HashMap, sync::Arc};

use tokio::{net::tcp::OwnedWriteHalf, sync::Mutex};

pub type ClientMap = Arc<Mutex<HashMap<String, Arc<Mutex<OwnedWriteHalf>>>>>;
