//The assemble function will take the Vec<Expr> as parameters.
pub type ControllerText = Vec<Expr>;
#[derive(Debug)]
pub enum Expr {
    Instruction(Instruction),
    Label(Label),
} 

#[derive(Debug)]
pub enum Instruction {
    Assign {target_reg: String, val_expr: ValueExpr},
    Test(OpreationExpr),
    Branch(Label),
    // Goto(PrimitiveExpr),
    // Save {reg: String},
    // Restore {reg: String},
    // Perform(OpreationExpr),
}
// impl Instruction {
//     fn make_instruction() // Construct instruction by combining resources
//     fn instruction_text(&self) -> String {
//     } // Implement as String output
//     fn instruction_execution_proc() // No need to construct closures anymore, directly change to execute corresponding procedure
// }
#[derive(Debug)]
pub enum ValueExpr {
    OpreationExpr(OpreationExpr),
    PrimitiveExpr(PrimitiveExpr),
}
#[derive(Debug)]
pub struct OpreationExpr {
    name: String,
    oprands: Vec<ValueExpr>,
    arity: usize,
}
impl OpreationExpr {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn oprands(&self) -> &[ValueExpr] {
        self.oprands.as_slice()
    }
    pub fn arity(&self) -> usize {
        self.arity
    }
}
#[derive(Debug)]
pub enum PrimitiveExpr {
    Constant(u32),
    Label(Label),
    Register(String),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label(String);
impl Label {
    pub fn get_name(&self) -> String {
        self.0.clone()
    }
}
pub fn parse(controller_text: &str) -> Result<(&str, ControllerText), String> {
    let mut remaining = controller_text.trim();
    let mut exprs = Vec::new();
    while !remaining.trim().is_empty() {
        let (new_remaining, expr) = parse_expr(remaining)?;
        exprs.push(expr);

        remaining = new_remaining;
    }
    Ok((remaining, exprs))
}

fn parse_expr(input: &str) -> Result<(&str, Expr), String> {
    let input = input.trim();
    if input.starts_with("(") {
        parse_instruction(input)
            .map(|(remaining,inst)| (remaining, Expr::Instruction(inst)))
    } else {
        parse_label(input)
            .map(|(remaining, label)| (remaining, Expr::Label(label)))
    }
}
fn parse_label(input: &str) -> Result<(&str, Label), String> {
    let (remaining, ident) = ident_parser(input.trim()).map_err(|_e| "Expected identifier for label")?;
    Ok((remaining, Label(ident.to_string())))
}
fn parse_instruction(input: &str) -> Result<(&str, Instruction), String> {
    let input = input.trim_start_matches("(").trim();
    if input.starts_with("assign") {
        parse_assign(input)
    } else if input.starts_with("test") {
        parse_test(input)
    } else if input.starts_with("branch") {
        parse_branch(input)
    } else {
        Err(format!("Failed to parse instruction: {}", 
            input.split_whitespace().next().unwrap_or("instruction not found.")
        ))
    }
}

// match a identifier and return it as a string
fn ident_parser(input: &str) -> Result<(&str, String), String> {
    let mut chars = input.chars();
    let mut ident = String::new();
    let is_allowed_in_ident= |c: char| -> bool  {
        matches!(c, '_' | '-' | '=' | '>' | '<' | '?' | '+' | '*' | '/' | '&' | '^' | '%' | '!')
    };
    while let Some(c) = chars.next() {
        if c.is_alphabetic() || is_allowed_in_ident(c) {
            ident.push(c);
        } else {
            break;
        }
    }
    if ident.is_empty() {
        Err("Expected identifier".to_string())
    } else {
        Ok((&input[ident.len()..].trim(), ident))
    }
}
fn number_parser(input: &str) -> Result<(&str, u32), String> {
    let mut chars = input.chars();
    let mut num = String::new();
    while let Some(c) = chars.next() {
        if c.is_numeric() {
            num.push(c);
        } else {
            break;
        }
    }
    if num.is_empty() {
        Err("Expected Value".to_string())
    } else {
        let value = num.parse()
            .map_err(|e| format!("Failed to parse the value '{num}' : {e}",))?;
        Ok((&input[num.len()..].trim(), value))
    }
}
fn parse_reg(input: &str) -> Result<(&str, String), String> {
    let input = input.trim_start_matches("reg").trim();

    let (input, name) = ident_parser(input).map_err(|e| {
        format!("{e:?}: register expression expects a name after 'reg' like (reg name)")
    })?;
    let input = input.starts_with(')')
        .then(|| input[1..].trim())
        .ok_or("Expects a ')' at the end of the register expression")?;
    
    Ok((input, name))
}
fn parse_const(input: &str) -> Result<(&str, u32), String> {
    let input = input.trim_start_matches("const").trim();

    let (input, value) = number_parser(input).map_err(|e| {
        format!("{e:?}: constant expression expects a const value after 'constant' like (const value)")
    })?;
    let input = input.starts_with(')')
        .then(|| input[1..].trim())
        .ok_or("Expects a ')' at the end of the constant expression")?;
    
    Ok((input, value))
}
fn parse_label_expr(input: &str) -> Result<(&str, Label), String> {
    let input = input.trim_start_matches("label").trim();

    let (input, label) = ident_parser(input).map_err(|e| {
        format!("{e:?}: label expression expects a label identity after 'label' like (label ident)")
    })?;
    let input = input.starts_with(')')
        .then(|| input[1..].trim())
        .ok_or("Expects a ')' at the end of the constant expression")?;
    
    Ok((input, Label(label)))
}
fn parse_primitive_expr(input: &str) -> Result<(&str, PrimitiveExpr), String> {
    let input = input.trim_start_matches('(').trim();
    if input.starts_with("label") {
        parse_label_expr(input).map(|(remaining, label)| (remaining, PrimitiveExpr::Label(label)))
    } else if input.starts_with("reg") {
        parse_reg(input).map(|(remaining, reg)| (remaining, PrimitiveExpr::Register(reg)))
    } else if input.starts_with("const") {
        parse_const(input).map(|(remaining, value)| (remaining, PrimitiveExpr::Constant(value)))
    } else {
        Err("Invalid primitive expression".to_string())
    }
}

// Example to be parsed: "op add (reg a) (reg b))"
//     Notice that the beginning '(' is already stripped
fn parse_operation(input: &str) -> Result<(&str, OpreationExpr), String> {
    let input = input.trim_start_matches("op").trim();

    let (input, operation) = ident_parser(input).map_err(|e| {
        format!("{e:?}: operation expression expects a name after 'op' like (op operation)")
    })?;

    let input = input.starts_with(')')
        .then(|| input[1..].trim())
        .ok_or("Expects a ')' at the end of the constant expression")?;

    if let Some(mut input) = input.starts_with('(')
        .then(|| input[1..].trim()) {
            if input.starts_with("op") {
                Err("Nested op is not allowed!".to_string())
            } else {
                let (mut input, first_param) = parse_primitive_expr(input)?;
                let mut oprands = vec![ValueExpr::PrimitiveExpr(first_param)];
                while input.starts_with('(') {
                    let (new_input, primitive) = parse_primitive_expr(input)?;
                    input = new_input;
                    oprands.push(ValueExpr::PrimitiveExpr(primitive));
                }
                
                input.trim().strip_prefix(')')
                    .ok_or("Expects a ')' at the end of one expression including op".to_string())
                    .map(|remaining| (remaining, OpreationExpr { 
                        name: operation.to_string(),
                        arity: oprands.len(),
                        oprands
                    }))
                
            }
    } else {
        //The operation does not have any operands here.
        Ok((input, OpreationExpr { 
            name: operation.to_string(),
            oprands: Vec::new(),
            arity: 0
        }))
    }
}
fn parse_value_expr(input: &str) -> Result<(&str, ValueExpr), String> {
    let input = input.trim_start_matches('(').trim();
    if input.starts_with("op") {
        parse_operation(input)
            .map(|(remaining, op)| (remaining, ValueExpr::OpreationExpr(op)))
    } else if input.starts_with("reg") {
        parse_reg(input).and_then(|(remaining, reg)| {
            if let Some(remaining) = remaining.strip_prefix(')') {
                Ok((remaining, ValueExpr::PrimitiveExpr(PrimitiveExpr::Register(reg))))
            } else {
                Err("Expects a ')' at the end of the assign expression".to_string())
            }
        })
    } else if input.starts_with("const") {
        parse_const(input).and_then(|(remaining, value)| {
            if let Some(remaining) = remaining.strip_prefix(')') {
                Ok((remaining, ValueExpr::PrimitiveExpr(PrimitiveExpr::Constant(value))))
            } else {
                Err("Expects a ')' at the end of the assign expression".to_string())
            }
        })
    } else if input.starts_with("label") {
        parse_label(input).and_then(|(remaining, label)| {
            if let Some(remaining) = remaining.strip_prefix(')') {
                Ok((remaining, ValueExpr::PrimitiveExpr(PrimitiveExpr::Label(label))))
            } else {
                Err("Expects a ')' at the end of the assign expression".to_string())
            }
        })
    } else {
        Err("Invalid values".to_string())
    }
}

fn parse_assign(input: &str) -> Result<(&str, Instruction), String> {
    let input = input.trim_start_matches("assign").trim();
    let (target, input ) = input.split_once(' ')
        .ok_or("Assign expects a target register")?;
    let input = input.trim();

    if !input.starts_with("(") { return Err("Assign expects a value".to_string()); };
    let (input , val) = parse_value_expr(input)
        .map_err(|e| format!("Invalid value expression: {}", e))?;

    Ok((input, Instruction::Assign { 
        target_reg: target.to_string(), 
        val_expr: val
    }))
}

// Example to be parsed: " test (op =) (reg b) (const 0)) "
// Notice that the beginning '(' is already stripped and 
//     the trailing ')' will be stripped by parse_operation().
fn parse_test(input: &str) -> Result<(&str, Instruction), String> {
    let input = input.trim_start_matches("test").trim();
    match input.strip_prefix('(') { 
        None => Err("Test expects an operation verrifying condition".to_string()),
        Some(input) => {
            let (input, cond) = parse_operation(input)?;
            Ok((input.trim(), Instruction::Test(cond)))
        }
    }
}

fn parse_branch(input: &str) -> Result<(&str, Instruction), String> {
    let input = input.trim_start_matches("branch").trim();
    match input.strip_prefix('(') { 
        None => Err("Branch expression expects a label".to_string()),
        Some(input) => {  
            match ident_parser(input) {
                Err(e) => 
                    Err("The label in branch expression expects a lable tag like (label label_name)".to_string()),
                Ok((input, tag)) if tag != "label" => 
                    Err("The label in branch expression expects a lable tag like (label label_name)".to_string()),
                Ok((input, tag)) => {
                    let (input, label) = parse_label_expr(input.trim())?;
                    if let Some(input) = input.trim().strip_prefix(')') {
                        Ok((input, Instruction::Branch(label)))
                    } else {
                        Err("Expects a ')' at the end of the branch expression".to_string())
                    }
                }
            } 

            
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn test_parse_expr1() {
        let input = 
        "gcd
            (test (op =) (reg b) (const 0))
            (branch (label gcd-done))
            (assign t (op rem) (reg a) (reg b))
        gcd-done";

        match parse(input) {
            Ok((remaining, exprs)) => {
                println!("Parsed expressions: {:#?}", exprs);
                println!("Remaining input: '{}'", remaining);
            }
            Err(e) => println!("Error: {}", e),
        }
    }
}
