#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, String, symbol_short, Address};

// Student ID structure with essential student information
#[contracttype]
#[derive(Clone)]
pub struct StudentID {
    pub student_id: u64,            // Unique student identifier
    pub name: String,               // Student's full name
    pub program: String,            // Program/major of study
    pub enrollment_year: u64,       // Year of enrollment
    pub expiration_date: u64,       // Expiration timestamp
    pub is_active: bool,            // Current status of the ID
}

// Registry to keep track of all student IDs
#[contracttype]
#[derive(Clone)]
pub struct Registry {
    pub total_ids: u64,             // Total number of IDs issued
    pub active_ids: u64,            // Number of currently active IDs
    pub expired_ids: u64,           // Number of expired IDs
}

// For mapping student_id to StudentID struct
#[contracttype]
pub enum DataKey {
    StudentRecord(u64),             // Maps student_id to StudentID
}

// Constants for storage
const REGISTRY: Symbol = symbol_short!("REGISTRY");
const ID_COUNTER: Symbol = symbol_short!("ID_COUNT");
const ADMIN: Symbol = symbol_short!("ADMIN");

#[contract]
pub struct StudentIDContract;

#[contractimpl]
impl StudentIDContract {
    // Initialize the contract with an admin address
    pub fn initialize(env: Env, admin: Address) {
        // Ensure contract is not already initialized
        if env.storage().instance().has(&ADMIN) {
            panic!("Contract already initialized");
        }
        
        // Set the admin address
        env.storage().instance().set(&ADMIN, &admin);
        
        // Initialize the registry
        let registry = Registry {
            total_ids: 0,
            active_ids: 0,
            expired_ids: 0,
        };
        
        env.storage().instance().set(&REGISTRY, &registry);
        env.storage().instance().set(&ID_COUNTER, &0u64);
        
        log!(&env, "Contract initialized with admin: {}", admin);
    }
    
    // Issue a new student ID (admin only)
    pub fn issue_student_id(
        env: Env, 
        admin: Address,
        name: String, 
        program: String, 
        enrollment_year: u64,
        validity_period: u64
    ) -> u64 {
        // Verify admin
        let stored_admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        if stored_admin != admin {
            panic!("Only admin can issue student IDs");
        }
        
        // Get current counter and increment
        let mut counter: u64 = env.storage().instance().get(&ID_COUNTER).unwrap_or(0);
        counter += 1;
        
        // Get current timestamp and calculate expiration
        let current_time = env.ledger().timestamp();
        let expiration_date = current_time + validity_period;
        
        // Create student ID
        let student_id = StudentID {
            student_id: counter,
            name: name,
            program: program,
            enrollment_year: enrollment_year,
            expiration_date: expiration_date,
            is_active: true,
        };
        
        // Update registry
        let mut registry: Registry = env.storage().instance().get(&REGISTRY).unwrap();
        registry.total_ids += 1;
        registry.active_ids += 1;
        
        // Store the data
        env.storage().instance().set(&DataKey::StudentRecord(counter), &student_id);
        env.storage().instance().set(&REGISTRY, &registry);
        env.storage().instance().set(&ID_COUNTER, &counter);
        
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Student ID issued: {}", counter);
        
        return counter;
    }
    
    // Verify a student ID's validity
    pub fn verify_student_id(env: Env, student_id: u64) -> bool {
        let key = DataKey::StudentRecord(student_id);
        
        if !env.storage().instance().has(&key) {
            return false;
        }
        
        let record: StudentID = env.storage().instance().get(&key).unwrap();
        let current_time = env.ledger().timestamp();
        
        // Check if ID is active and not expired
        return record.is_active && record.expiration_date > current_time;
    }
    
    // Retrieve student information
    pub fn get_student_info(env: Env, student_id: u64) -> StudentID {
        let key = DataKey::StudentRecord(student_id);
        
        if !env.storage().instance().has(&key) {
            panic!("Student ID does not exist");
        }
        
        return env.storage().instance().get(&key).unwrap();
    }
}