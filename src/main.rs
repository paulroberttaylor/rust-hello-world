mod tree_sitter;
mod sfapex;

use tree_sitter::Parser;

fn main() {
    println!("=== Tree-sitter Salesforce Grammar Demo ===\n");

    // Parse some Apex code
    println!("1. Parsing APEX Code\n");
    let mut parser = Parser::new();
    parser.set_language(sfapex::apex::language()).unwrap();

    let apex_code = r#"
/**
 * Example Apex class
 */
global class AccountManager implements Database.Batchable<SObject> {
    public static String DEFAULT_NAME = 'Unknown';

    global Account createAccount(String name, String industry){
        Account acct = new Account();
        acct.Name = name;
        acct.Industry = industry;
        return acct;
    }

    global Database.QueryLocator start(Database.BatchableContext bc) {
        return Database.getQueryLocator('SELECT Id, Name FROM Account');
    }
}"#;

    let tree = parser.parse(apex_code).expect("Failed to parse Apex code");
    println!("Parse tree (S-expression):\n{}\n", tree.root_node().to_sexp());

    // Parse some SOQL
    println!("2. Parsing SOQL Query\n");
    parser.set_language(sfapex::soql::language()).unwrap();

    let soql_query = r#"
SELECT Id, Name, Parent.Name,
    TYPEOF Owner
        WHEN User THEN Id, Username, Email
        WHEN Group THEN Name, DeveloperName
    END,
    (SELECT Id, FirstName, LastName FROM Contacts)
FROM Account
WHERE Name = 'Acme' AND IsActive__c = TRUE
ORDER BY CreatedDate DESC
LIMIT 100"#;

    let tree = parser.parse(soql_query).expect("Failed to parse SOQL query");
    println!("Parse tree (S-expression):\n{}\n", tree.root_node().to_sexp());

    // Parse some SOSL
    println!("3. Parsing SOSL Query\n");
    parser.set_language(sfapex::sosl::language()).unwrap();

    let sosl_query = r#"
FIND {Acme}
IN ALL FIELDS
RETURNING Account(Id, Name), Contact(Id, FirstName, LastName)
LIMIT 50"#;

    let tree = parser.parse(sosl_query).expect("Failed to parse SOSL query");
    println!("Parse tree (S-expression):\n{}\n", tree.root_node().to_sexp());

    println!("=== All parsing completed successfully! ===");
}
