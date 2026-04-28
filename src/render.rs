use std::error::Error;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

fn count_visual_lines(line: &str, term_width: usize) -> usize {
    if line.is_empty() {
        return 1;
    }

    let mut lines = 0;
    let mut current_width = 0;

    for c in line.chars() {
        let w = c.width().unwrap_or(0);
        // 如果当前行放不下这个字符，先换行
        if current_width + w > term_width {
            lines += 1;
            current_width = w;
        } else {
            current_width += w;
        }
    }

    if current_width > 0 {
        lines += 1;
    }

    lines
}

pub fn count_lines_in_terminal(text: &str, term_width: usize) -> Result<usize, Box<dyn Error>> {
    let lines = text.lines();
    let mut result = if text.width() == 0 {
        0
    } else {
        lines.map(|line| count_visual_lines(line, term_width)).sum()
    };
    if text.ends_with('\n') {
        result += 1;
    }
    Ok(result)
}

pub fn clear_lines_above(n: usize) {
    print!("\n\x1B[{n}A\x1B[G\x1B[0J");
}

pub fn render_markdown(text: &str) {
    let skin = termimad::MadSkin::default();
    println!("{}", skin.term_text(text));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string_returns_zero() {
        assert_eq!(count_lines_in_terminal("", 80).unwrap(), 0);
    }

    #[test]
    fn single_line_no_newline() {
        assert_eq!(count_lines_in_terminal("hello", 80).unwrap(), 1);
    }

    #[test]
    fn trailing_newline_counts_as_extra_line() {
        assert_eq!(count_lines_in_terminal("hello\n", 80).unwrap(), 2);
    }

    #[test]
    fn multiple_physical_lines() {
        assert_eq!(count_lines_in_terminal("a\nb\nc", 80).unwrap(), 3);
    }

    #[test]
    fn many_newlines() {
        assert_eq!(count_lines_in_terminal("\n\n\n\n", 80).unwrap(), 5);
    }

    #[test]
    fn long_line_wraps() {
        let text = "a".repeat(100) + "\n";
        assert_eq!(count_lines_in_terminal(&text, 80).unwrap(), 3);
    }

    #[test]
    fn cjk_wrap() {
        assert_eq!(
            count_lines_in_terminal("天地玄黄\n\n宇宙洪荒", 2).unwrap(),
            9
        );
        assert_eq!(count_lines_in_terminal("天地玄黄宇宙洪荒", 3).unwrap(), 8);
    }

    const COMPLICATE_TEXT: &str = r#"
你好！下面我为你提供一段具有教学示范意义的 Rust 代码，这段代码展示了 Rust 的多个核心特性：

```rust
/// 一个教学示例：学生成绩管理系统
/// 展示 Rust 的核心特性：所有权、借用、模式匹配、错误处理、Trait 等

use std::collections::HashMap;

// 定义枚举表示成绩等级
#[derive(Debug, Clone, Copy, PartialEq)]
enum Grade {
    A,
    B,
    C,
    D,
    F,
}

// 为 Grade 实现 Display trait，使其可以被格式化输出
impl std::fmt::Display for Grade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Grade::A => write!(f, "A"),
            Grade::B => write!(f, "B"),
            Grade::C => write!(f, "C"),
            Grade::D => write!(f, "D"),
            Grade::F => write!(f, "F"),
        }
    }
}

// 定义自定义错误类型
#[derive(Debug)]
enum StudentError {
    StudentNotFound(String),
    InvalidScore(String),
    AlreadyExists(String),
}

impl std::fmt::Display for StudentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StudentError::StudentNotFound(name) => {
                write!(f, "学生不存在: {}", name)
            }
            StudentError::InvalidScore(msg) => {
                write!(f, "无效的成绩: {}", msg)
            }
            StudentError::AlreadyExists(name) => {
                write!(f, "学生已存在: {}", name)
            }
        }
    }
}

impl std::error::Error for StudentError {}

// 定义学生结构体
#[derive(Debug, Clone)]
struct Student {
    id: u32,
    name: String,
    scores: HashMap<String, u8>,
}

// 为 Student 实现方法
impl Student {
    fn new(id: u32, name: &str) -> Self {
        Student {
            id,
            name: name.to_string(),
            scores: HashMap::new(),
        }
    }

    // 添加成绩 - 展示借用和返回值
    fn add_score(&mut self, subject: &str, score: u8) -> Result<(), StudentError> {
        if score > 100 {
            return Err(StudentError::InvalidScore(format!(
                "分数 {} 超过 100",
                score
            )));
        }
        self.scores.insert(subject.to_string(), score);
        Ok(())
    }

    // 获取某门课程的等级 - 展示模式匹配
    fn get_grade(&self, subject: &str) -> Option<Grade> {
        let score = self.scores.get(subject)?;
        
        match score {
            90..=100 => Some(Grade::A),
            80..=89  => Some(Grade::B),
            70..=79  => Some(Grade::C),
            60..=69  => Some(Grade::D),
            _        => Some(Grade::F),
        }
    }

    // 获取平均成绩 - 展示迭代器链
    fn average_score(&self) -> Option<f64> {
        if self.scores.is_empty() {
            return None;
        }
        
        let sum: u64 = self.scores.values().map(|s| *s as u64).sum();
        Some(sum as f64 / self.scores.len() as f64)
    }

    // 判断是否及格 - 展示闭包和 filter
    fn is_passing(&self, min_passing_score: u8) -> bool {
        self.scores
            .values()
            .all(|&score| score >= min_passing_score)
    }
}

// 定义学生管理类
struct StudentManager {
    students: HashMap<u32, Student>,
    next_id: u32,
}

impl StudentManager {
    fn new() -> Self {
        StudentManager {
            students: HashMap::new(),
            next_id: 1,
        }
    }

    // 添加学生 - 展示可变借用和插入
    fn add_student(&mut self, name: &str) -> Result<u32, StudentError> {
        // 检查是否已存在同名学生
        if self.students.values().any(|s| s.name == name) {
            return Err(StudentError::AlreadyExists(name.to_string()));
        }
        
        let id = self.next_id;
        let student = Student::new(id, name);
        self.next_id += 1;
        self.students.insert(id, student);
        Ok(id)
    }

    // 修改学生成绩 - 展示可变借用
    fn add_student_score(
        &mut self,
        student_id: u32,
        subject: &str,
        score: u8,
    ) -> Result<(), StudentError> {
        let student = self.students.get_mut(&student_id)
            .ok_or_else(|| StudentError::StudentNotFound(student_id.to_string()))?;
        
        student.add_score(subject, score)
    }

    // 获取学生信息 - 展示不可变借用
    fn get_student(&self, student_id: u32) -> Result<&Student, StudentError> {
        self.students.get(&student_id)
            .ok_or_else(|| StudentError::StudentNotFound(student_id.to_string()))
    }

    // 打印所有学生信息
    fn print_all_students(&self) {
        println!("\n========== 学生列表 ==========");
        for (id, student) in &self.students {
            println!("ID: {}, 姓名: {}", id, student.name);
            
            for (subject, score) in &student.scores {
                if let Some(grade) = student.get_grade(subject) {
                    println!("  {}: {}分 ({})", subject, score, grade);
                }
            }
            
            if let Some(avg) = student.average_score() {
                println!("  平均分: {:.2}", avg);
            }
            
            println!("  是否全部及格: {}", student.is_passing(60));
        }
        println!("==============================\n");
    }
}

// 实现 Drop trait，展示析构函数
impl Drop for StudentManager {
    fn drop(&mut self) {
        println!("学生管理系统已关闭");
    }
}

fn main() {
    println!("欢迎来到学生成绩管理系统！\n");

    // 创建管理器
    let mut manager = StudentManager::new();

    // 添加学生
    let alice_id = manager.add_student("Alice").unwrap();
    let bob_id = manager.add_student("Bob").unwrap();
    let charlie_id = manager.add_student("Charlie").unwrap();

    println!("成功添加学生: Alice, Bob, Charlie\n");

    // 添加成绩
    let add_scores = [
        (alice_id, "数学", 95),
        (alice_id, "英语", 88),
        (alice_id, "物理", 76),
        (bob_id, "数学", 72),
        (bob_id, "英语", 65),
        (bob_id, "物理", 58),
        (charlie_id, "数学", 85),
        (charlie_id, "英语", 92),
        (charlie_id, "物理", 78),
    ];

    println!("正在添加成绩...");
    for (id, subject, score) in &add_scores {
        match manager.add_student_score(*id, subject, *score) {
            Ok(()) => println!("  ✓ {} - {}: {}分", id, subject, score),
            Err(e) => println!("  ✗ 错误: {}", e),
        }
    }

    // 处理错误示例
    println!("\n尝试添加无效成绩（错误处理示例）:");
    match manager.add_student_score(alice_id, "化学", 150) {
        Ok(()) => println!("  成绩添加成功"),
        Err(e) => println!("  ✗ {}", e),
    }

    match manager.add_student_score(999, "数学", 80) {
        Ok(()) => println!("  成绩添加成功"),
        Err(e) => println!("  ✗ {}", e),
    }

    // 打印所有学生信息
    manager.print_all_students();

    // 演示不可变借用
    if let Ok(alice) = manager.get_student(alice_id) {
        println!("Alice 的成绩详情:");
        for (subject, score) in &alice.scores {
            println!("  {}: {}分", subject, score);
        }
    }
}
```

## 这段代码展示的核心 Rust 特性：

1. **所有权和借用**：`&self`（不可变借用）、`&mut self`（可变借用）
2. **枚举和模式匹配**：`Grade` 枚举和 `match` 语句
3. **错误处理**：自定义错误类型和 `Result` 类型
4. **Trait 实现**：`Display`、`Debug`、`Drop` trait
5. **数据结构**：`HashMap` 的使用
6. **迭代器**：链式调用、`filter`、`all`、`map`、`sum`
7. **Option 类型**：安全的空值处理
8. **闭包**：匿名函数的使用
9. **宏和 derive**：`#[derive(Debug, Clone, Copy)]`

你可以直接运行这段代码：
```bash
cargo run
```
"#;

    #[test]
    fn complicate_text_nowrap() {
        assert_eq!(count_lines_in_terminal(COMPLICATE_TEXT, 1000).unwrap(), 271);
    }
}
