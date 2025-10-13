use colored::*;
use serde::{Deserialize, Serialize};
use sled::{Db};
use std::collections::HashMap;
use std::io::{self, Write};
use std::process::exit;

// === Estruturas de dados ===
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

// === Funções utilitárias ===
fn input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    buf.trim().to_string()
}

fn serialize<T: Serialize>(value: &T) -> Vec<u8> {
    bincode::serialize(value).expect("Erro ao serializar")
}

fn deserialize<'a, T: Deserialize<'a>>(bytes: &'a [u8]) -> T {
    bincode::deserialize(bytes).expect("Erro ao desserializar")
}

// === Funções de persistência com Sled ===
fn open_db() -> Db {
    sled::open("kanban_db").expect("Erro ao abrir banco de dados Sled")
}

fn save_project(db: &Db, name: &str, proj: &Project) {
    db.insert(name.as_bytes(), serialize(proj))
        .expect("Erro ao salvar projeto");
    db.flush().unwrap();
}

fn load_all_projects(db: &Db) -> HashMap<String, Project> {
    let mut map = HashMap::new();
    for item in db.iter() {
        let (k, v) = item.expect("Erro ao ler item do banco");
        let key = String::from_utf8(k.to_vec()).unwrap();
        let proj: Project = deserialize(&v);
        map.insert(key, proj);
    }
    map
}

fn delete_project(db: &Db, name: &str) {
    db.remove(name.as_bytes()).unwrap();
    db.flush().unwrap();
}

// === Funções de negócio ===
fn create_project(db: &Db) {
    let nome = input("Nome do novo projeto: ");
    let tipo = input("Tipo ('kanban' ou 'todo'): ").to_lowercase();

    if db.contains_key(nome.as_bytes()).unwrap() {
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

    let proj = Project {
        project_type: tipo,
        data,
    };

    save_project(db, &nome, &proj);
    println!("Projeto '{}' criado com sucesso.", nome);
}

fn list_projects(db: &Db) {
    let mut i = 1;
    println!("\n=== Projetos ===");
    for item in db.iter() {
        let (k, v) = item.unwrap();
        let key = String::from_utf8(k.to_vec()).unwrap();
        let proj: Project = deserialize(&v);
        println!("{}. {} ({})", i, key, proj.project_type);
        i += 1;
    }
    if i == 1 {
        println!("Nenhum projeto criado ainda.");
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

fn enter_project(db: &Db, name: &str, mut proj: Project) {
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
            "exit" => {
                save_project(db, name, &proj);
                break;
            }
            _ => println!("Comando inválido."),
        }
    }
}

// === Main ===
fn main() {
    let db = open_db();
    println!("=== Gerenciador de Kanbans e To-Do Lists (Sled) ===");

    loop {
        println!("\nComandos globais:");
        println!("  criar   -> cria novo projeto");
        println!("  mostrar -> lista todos os projetos");
        println!("  abrir   -> abre projeto existente");
        println!("  apagar  -> remove projeto");
        println!("  sair    -> fecha o app");

        let cmd = input(">> ").to_lowercase();
        match cmd.as_str() {
            "criar" => create_project(&db),
            "mostrar" => list_projects(&db),
            "abrir" => {
                list_projects(&db);
                let nome = input("Digite o nome do projeto: ");
                if let Some(v) = db.get(nome.as_bytes()).unwrap() {
                    let proj: Project = deserialize(&v);
                    enter_project(&db, &nome, proj);
                } else {
                    println!("Projeto não encontrado.");
                }
            }
            "apagar" => {
                let nome = input("Nome do projeto a remover: ");
                delete_project(&db, &nome);
                println!("Projeto removido.");
            }
            "sair" => {
                db.flush().unwrap();
                exit(0);
            }
            _ => println!("Comando inválido."),
        }
    }
}
