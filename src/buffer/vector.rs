pub struct VectorBuffer {
    buffer: Vec<String>,
}

impl VectorBuffer {
    pub fn from_string(str: &String) -> std::io::Result<VectorBuffer> {
        let mut vector = Vec::new();
        for line in str.lines() {
            vector.push(line.to_string());
        }
        Ok(VectorBuffer { buffer: vector })
    }

    pub fn to_string(&self) -> std::io::Result<String> {
        let mut str = String::new();
        for line in self.buffer.iter() {
            str.push_str(line.as_str());
            str.push('\n');
        }
        Ok(str)
    }

    pub fn insert_char(&mut self, input: char, x: usize, y: usize) {
        if input == '\n' {
            //1. from col find the remaining string to be trasnported to next line
            let removed: String = self.buffer[y - 1].drain(x..).collect();
            //2. create new string and insert at row+1 in vector
            self.buffer.insert(y, removed)
        } else {
            self.buffer[y - 1].insert(x, input);
        }
    }

    pub fn remove_char(&mut self, x: usize, y: usize) {
        if x > 0 {
            self.buffer[y].remove(x - 1);
        } else {
            if y > 0 {
                let line = self.buffer.remove(y);
                self.buffer[y - 1].push_str(&line);
            }
        }
    }

    //get ith line string
    pub fn get_line(&self, i: usize) -> &String {
        return &self.buffer[i];
    }

    pub fn size(&self) -> usize {
        return self.buffer.len();
    }
}
