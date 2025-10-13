use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use std::process::exit;

const DB_FILE: &str = "kanban_db.json";

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Task {
    title: String,
    subtasks: Vec<String>,
    subkanban: Option<HashMap<String, Vec<Task>>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Project {
    #[serde(rename = "type")]
    project_type: String,
    data: HashMap<String, Vec<Task>>,
}

type Database = HashMap<String, Project>;

fn load_db() -> Database {
    if Path::new(DB_FILE).exists() {
        let data = fs::read_to_string(DB_FILE).expect("Erro ao ler o arquivo");
        serde_json::from_str(&data).unwrap_or_else(|_| {
            eprintln!("Erro ao parsear o JSON.");
            HashMap::new()
        })
    } else {
        HashMap::new()
    }
}

fn save_db(db: &Database) {
    let json = serde_json::to_string_pretty(db).expect("Erro ao converter para JSON");
    let mut file = File::create(DB_FILE).expect("Erro ao criar arquivo");
    file.write_all(json.as_bytes()).expect("Erro ao salvar");
}

fn input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    buf.trim().to_string()
}

fn create_project(db: &mut Database) {
    let nome = input("Nome do novo projeto: ");
    let tipo = input("Tipo ('kanban' ou 'todo'): ").to_lowercase();

    if db.contains_key(&nome) {
        println!("Já existe um projeto com esse nome.");
        return;
    }

    let mut data = HashMap::new();

    match tipo.as_str() {
        "kanban" => {
            data.insert("A Fazer".into(), Vec::new());
            data.insert("Em Progresso".into(), Vec::new());
            data.insert("Concluído".into(), Vec::new());
        }
        "todo" => {
            data.insert("ToDo".into(), Vec::new());
            data.insert("Feito".into(), Vec::new());
        }
        _ => {
            println!("Tipo inválido.");
            return;
        }
    }

    db.insert(
        nome.clone(),
        Project {
            project_type: tipo.clone(),
            data,
        },
    );
    save_db(db);
    println!("Projeto '{}' criado com sucesso.", nome);
}

fn list_projects(db: &Database) {
    if db.is_empty() {
        println!("Nenhum projeto criado ainda.");
        return;
    }
    println!("\n=== Projetos ===");
    for (i, (name, proj)) in db.iter().enumerate() {
        println!("{}. {} ({})", i + 1, name, proj.project_type);
    }
    println!();
}

fn show(board: &HashMap<String, Vec<Task>>, indent: usize) {
    for (col, tasks) in board {
        println!("{}[{}]", " ".repeat(indent), col.blue());
        for (i, task) in tasks.iter().enumerate() {
            println!("{}{}. {}", " ".repeat(indent + 2), i + 1, task.title);
            for sub in &task.subtasks {
                println!("{}- {}", " ".repeat(indent + 5), sub);
            }
            if task.subkanban.is_some() {
                println!("{}(Tem sub-kanban)", " ".repeat(indent + 5));
            }
        }
    }
    println!();
}

fn add(board: &mut HashMap<String, Vec<Task>>, col: &str, title: &str) {
    if !board.contains_key(col) {
        println!("Coluna inválida.");
        return;
    }
    board.get_mut(col).unwrap().push(Task {
        title: title.to_string(),
        subtasks: Vec::new(),
        subkanban: None,
    });
    println!("Tarefa '{}' adicionada em '{}'.", title, col);
}

fn add_subtask(board: &mut HashMap<String, Vec<Task>>, col: &str, idx: usize, subt: &str) {
    if let Some(tasks) = board.get_mut(col) {
        if let Some(task) = tasks.get_mut(idx - 1) {
            task.subtasks.push(subt.to_string());
            println!("Subtarefa adicionada.");
        } else {
            println!("Índice inválido.");
        }
    } else {
        println!("Coluna inválida.");
    }
}

fn move_task(board: &mut HashMap<String, Vec<Task>>, c1: &str, c2: &str, idx: usize) {
    if let Some(from_col) = board.get_mut(c1) {
        if idx == 0 || idx > from_col.len() {
            println!("Índice inválido.");
            return;
        }
        let task = from_col.remove(idx - 1);
        if let Some(to_col) = board.get_mut(c2) {
            to_col.push(task);
            println!("Tarefa movida.");
        } else {
            println!("Coluna destino inválida.");
        }
    } else {
        println!("Coluna origem inválida.");
    }
}

fn delete_task(board: &mut HashMap<String, Vec<Task>>, col: &str, idx: usize) {
    if let Some(tasks) = board.get_mut(col) {
        if idx == 0 || idx > tasks.len() {
            println!("Índice inválido.");
            return;
        }
        let t = tasks.remove(idx - 1);
        println!("Tarefa '{}' removida.", t.title);
    } else {
        println!("Coluna inválida.");
    }
}

fn enter_subkanban(task: &mut Task) {
    let board = task.subkanban.as_mut().unwrap();
    println!("\nEntrando no subprojeto de '{}'", task.title);
    loop {
        let cmd = input(&format!("({}) >> ", task.title));
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        match parts[0] {
            "show" => show(board, 2),
            "add" if parts.len() >= 3 => add(board, parts[1], parts[2]),
            "add_sub" if parts.len() >= 4 => add_subtask(board, parts[1], parts[2].parse().unwrap(), parts[3]),
            "move" if parts.len() >= 4 => move_task(board, parts[1], parts[2], parts[3].parse().unwrap()),
            "del" if parts.len() >= 3 => delete_task(board, parts[1], parts[2].parse().unwrap()),
            "exit" => break,
            _ => println!("Comando inválido."),
        }
    }
}

fn open_subkanban(task: &mut Task) {
    if task.subkanban.is_none() {
        let tipo = input("Criar 'kanban' ou 'todo'? ").to_lowercase();
        let mut board = HashMap::new();
        match tipo.as_str() {
            "kanban" => {
                board.insert("A Fazer".into(), Vec::new());
                board.insert("Em Progresso".into(), Vec::new());
                board.insert("Concluído".into(), Vec::new());
            }
            "todo" => {
                board.insert("ToDo".into(), Vec::new());
                board.insert("Feito".into(), Vec::new());
            }
            _ => {
                println!("Tipo inválido.");
                return;
            }
        }
        task.subkanban = Some(board);
    }
    enter_subkanban(task);
}

fn enter_project(name: &str, proj: &mut Project) {
    println!("\n=== Projeto: {} ({}) ===", name, proj.project_type);
    println!("Comandos: show, add, add_sub, move, del, open, exit\n");
    loop {
        let cmd = input(&format!("({}) >> ", name));
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        match parts[0] {
            "show" => show(&proj.data, 0),
            "add" if parts.len() >= 3 => add(&mut proj.data, parts[1], parts[2]),
            "add_sub" if parts.len() >= 4 => add_subtask(&mut proj.data, parts[1], parts[2].parse().unwrap(), parts[3]),
            "move" if parts.len() >= 4 => move_task(&mut proj.data, parts[1], parts[2], parts[3].parse().unwrap()),
            "del" if parts.len() >= 3 => delete_task(&mut proj.data, parts[1], parts[2].parse().unwrap()),
            "open" if parts.len() >= 3 => {
                let col = parts[1];
                let idx: usize = parts[2].parse().unwrap();
                if let Some(tasks) = proj.data.get_mut(col) {
                    if let Some(task) = tasks.get_mut(idx - 1) {
                        open_subkanban(task);
                    } else {
                        println!("Índice inválido.");
                    }
                } else {
                    println!("Coluna inválida.");
                }
            }
            "exit" => break,
            _ => println!("Comando inválido."),
        }
    }
}

fn main() {
    let mut db = load_db();
    println!("=== Gerenciador de Kanbans e To-Do Lists ===");

    loop {
        println!("\nComandos globais:");
        println!("  criar   -> cria novo projeto");
        println!("  mostrar -> lista todos os projetos");
        println!("  abrir   -> abre projeto existente");
        println!("  sair    -> fecha o app");

        let cmd = input(">> ").to_lowercase();
        match cmd.as_str() {
            "criar" => create_project(&mut db),
            "mostrar" => list_projects(&db),
            "abrir" => {
                list_projects(&db);
                let nome = input("Digite o nome do projeto: ");
                if let Some(proj) = db.get_mut(&nome) {
                    enter_project(&nome, proj);
                    save_db(&db);
                } else {
                    println!("Projeto não encontrado.");
                }
            }
            "sair" => {
                save_db(&db);
                exit(0);
            }
            _ => println!("Comando inválido."),
        }
    }
}
