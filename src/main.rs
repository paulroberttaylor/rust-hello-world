mod tree_sitter;
mod sfapex;

use tree_sitter::{Parser, Query, QueryCursor};

fn main() {
    println!("=== Tree-sitter Salesforce Grammar Demo ===\n");

    // Parse some Apex code
    println!("1. Parsing APEX Code\n");
    let mut parser = Parser::new();
    let apex_lang = sfapex::apex::language();
    parser.set_language(apex_lang).unwrap();

    let apex_code = r#"
/**
 * Complex Order Management System
 * Demonstrates multiple features: fields, methods, inner classes, exceptions, enums
 */
public class OrderManagementService {
    // Static constants
    private static final String DEFAULT_CURRENCY = 'USD';
    private static final Integer MAX_LINE_ITEMS = 100;
    private static final Decimal TAX_RATE = 0.085;

    // Instance fields
    private Map<Id, Order> orderCache;
    private List<OrderValidator> validators;
    public Integer processedCount { get; private set; }

    // Static field with initialization
    private static OrderManagementService instance;

    /**
     * Enum for order status
     */
    public enum OrderStatus {
        DRAFT, PENDING, APPROVED, SHIPPED, DELIVERED, CANCELLED
    }

    /**
     * Inner class representing order configuration
     */
    public class OrderConfig {
        public String region { get; set; }
        public Boolean autoApprove { get; set; }
        public Decimal discountRate { get; set; }

        public OrderConfig(String region) {
            this.region = region;
            this.autoApprove = false;
            this.discountRate = 0.0;
        }
    }

    /**
     * Custom exception class
     */
    public class OrderException extends Exception {
        private String errorCode;

        public OrderException(String message, String code) {
            this.setMessage(message);
            this.errorCode = code;
        }

        public String getErrorCode() {
            return this.errorCode;
        }
    }

    /**
     * Singleton pattern constructor
     */
    private OrderManagementService() {
        this.orderCache = new Map<Id, Order>();
        this.validators = new List<OrderValidator>();
        this.processedCount = 0;
    }

    /**
     * Get singleton instance
     */
    public static OrderManagementService getInstance() {
        if (instance == null) {
            instance = new OrderManagementService();
        }
        return instance;
    }

    /**
     * Process multiple orders with exception handling
     */
    public List<Order> processOrders(List<Order> orders, OrderConfig config) {
        List<Order> processedOrders = new List<Order>();

        try {
            validateConfig(config);

            for (Order ord : orders) {
                if (ord.LineItems.size() > MAX_LINE_ITEMS) {
                    throw new OrderException(
                        'Too many line items: ' + ord.LineItems.size(),
                        'ERR_LINE_ITEMS'
                    );
                }

                // Calculate totals
                Decimal subtotal = 0;
                for (OrderItem item : ord.LineItems) {
                    subtotal += item.UnitPrice * item.Quantity;
                }

                Decimal tax = subtotal * TAX_RATE;
                Decimal discount = subtotal * config.discountRate;
                ord.TotalAmount = subtotal + tax - discount;

                // Auto-approve if configured
                if (config.autoApprove && ord.TotalAmount < 10000) {
                    ord.Status = String.valueOf(OrderStatus.APPROVED);
                } else {
                    ord.Status = String.valueOf(OrderStatus.PENDING);
                }

                processedOrders.add(ord);
                orderCache.put(ord.Id, ord);
            }

            update processedOrders;
            this.processedCount += processedOrders.size();

        } catch (OrderException e) {
            System.debug('Order validation failed: ' + e.getMessage());
            throw e;
        } catch (DmlException e) {
            System.debug('Database error: ' + e.getMessage());
            rollbackOrders(processedOrders);
            throw new OrderException('Failed to save orders', 'ERR_DML');
        } catch (Exception e) {
            System.debug('Unexpected error: ' + e.getMessage());
            throw new OrderException('System error occurred', 'ERR_SYSTEM');
        } finally {
            logProcessing(processedOrders.size(), config.region);
        }

        return processedOrders;
    }

    /**
     * Private validation method
     */
    private void validateConfig(OrderConfig config) {
        if (config == null) {
            throw new OrderException('Configuration cannot be null', 'ERR_CONFIG');
        }

        if (String.isBlank(config.region)) {
            throw new OrderException('Region is required', 'ERR_REGION');
        }

        if (config.discountRate < 0 || config.discountRate > 0.5) {
            throw new OrderException('Invalid discount rate', 'ERR_DISCOUNT');
        }
    }

    /**
     * Query orders by status with SOQL
     */
    public List<Order> getOrdersByStatus(OrderStatus status) {
        String statusStr = String.valueOf(status);

        return [
            SELECT Id, OrderNumber, Status, TotalAmount,
                   (SELECT Id, Product2.Name, Quantity, UnitPrice FROM OrderItems)
            FROM Order
            WHERE Status = :statusStr
            ORDER BY CreatedDate DESC
            LIMIT 1000
        ];
    }

    /**
     * Private helper method with while loop
     */
    private void rollbackOrders(List<Order> orders) {
        Integer index = 0;
        while (index < orders.size()) {
            Order ord = orders[index];
            if (orderCache.containsKey(ord.Id)) {
                orderCache.remove(ord.Id);
            }
            index++;
        }
    }

    /**
     * Static utility method
     */
    public static String formatCurrency(Decimal amount) {
        return DEFAULT_CURRENCY + ' ' + amount.setScale(2).format();
    }

    /**
     * Logging method
     */
    @future
    private static void logProcessing(Integer count, String region) {
        System.debug('Processed ' + count + ' orders in region: ' + region);
    }
}"#;

    let tree = parser.parse(apex_code).expect("Failed to parse Apex code");

    // Extract semantic information using queries
    println!("Extracting class and method information...\n");
    extract_apex_info(&tree, apex_lang, apex_code);

    // Extract and parse embedded SOQL queries
    println!("Extracting embedded SOQL queries...\n");
    extract_soql_from_apex(&tree, apex_lang, apex_code);

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

fn extract_apex_info(tree: &tree_sitter::Tree, language: *const std::ffi::c_void, source: &str) {
    // Query for class declarations
    let class_query_str = r#"
        (class_declaration
            name: (identifier) @class.name) @class.definition

        (enum_declaration
            name: (identifier) @enum.name) @enum.definition
    "#;

    // Query for method declarations with more details
    let method_query_str = r#"
        (method_declaration
            type: (_) @method.return_type
            name: (identifier) @method.name
            parameters: (formal_parameters) @method.parameters) @method.definition

        (constructor_declaration
            name: (identifier) @constructor.name
            parameters: (formal_parameters) @constructor.parameters) @constructor.definition
    "#;

    // Execute class query
    let class_query = Query::new(language, class_query_str)
        .expect("Failed to create class query");
    let mut cursor = QueryCursor::new();
    cursor.exec(&class_query, &tree.root_node());

    println!("Classes and Enums found:");
    println!("========================");
    while let Some(captures) = cursor.next_match(&class_query) {
        for (name, node) in captures {
            if name.contains("class.name") || name.contains("enum.name") {
                let text = node.utf8_text(source);
                let kind = if name.contains("enum") { "Enum" } else { "Class" };
                println!("  {} {}: {}", kind, name, text);
            }
        }
    }

    // Execute method query
    let method_query = Query::new(language, method_query_str)
        .expect("Failed to create method query");
    let mut cursor = QueryCursor::new();
    cursor.exec(&method_query, &tree.root_node());

    println!("\nMethods and Constructors found:");
    println!("================================");

    while let Some(captures) = cursor.next_match(&method_query) {
        let mut method_name = String::new();
        let mut return_type = String::new();
        let mut params = String::new();
        let mut is_constructor = false;

        // Process all captures in this match
        for (name, node) in captures {
            match name.as_str() {
                "method.name" => {
                    method_name = node.utf8_text(source).to_string();
                }
                "constructor.name" => {
                    method_name = node.utf8_text(source).to_string();
                    is_constructor = true;
                }
                "method.return_type" => {
                    return_type = node.utf8_text(source).to_string();
                }
                "method.parameters" | "constructor.parameters" => {
                    params = node.utf8_text(source).to_string();
                }
                _ => {}
            }
        }

        // Print this match
        if !method_name.is_empty() {
            if is_constructor {
                println!("  Constructor: {}{}", method_name, params);
            } else {
                println!("  Method: {} {}{}", return_type, method_name, params);
            }
        }
    }

    println!();
}

fn extract_soql_from_apex(tree: &tree_sitter::Tree, language: *const std::ffi::c_void, source: &str) {
    // Query for embedded SOQL queries
    let soql_query_str = r#"
        (query_expression
            (soql_query_body) @soql.query) @soql.expression
    "#;

    let query = Query::new(language, soql_query_str)
        .expect("Failed to create SOQL query");
    let mut cursor = QueryCursor::new();
    cursor.exec(&query, &tree.root_node());

    let mut query_count = 0;

    while let Some(captures) = cursor.next_match(&query) {
        for (name, node) in captures {
            if name == "soql.query" {
                query_count += 1;
                let soql_text = node.utf8_text(source);

                println!("Embedded SOQL Query #{}:", query_count);
                println!("------------------------");
                println!("{}", soql_text);

                // Now parse this SOQL with the SOQL grammar
                let mut soql_parser = Parser::new();
                let soql_lang = sfapex::soql::language();
                soql_parser.set_language(soql_lang).unwrap();

                if let Some(soql_tree) = soql_parser.parse(soql_text) {
                    println!("\nSOQL Structure:");
                    analyze_soql(&soql_tree, soql_lang, soql_text);
                }
                println!();
            }
        }
    }

    if query_count == 0 {
        println!("  No embedded SOQL queries found.");
    }
    println!();
}

fn analyze_soql(tree: &tree_sitter::Tree, language: *const std::ffi::c_void, source: &str) {
    // Query to extract SOQL components
    let soql_analysis_query = r#"
        (select_clause
            (field_identifier) @field)

        (from_clause
            (storage_identifier) @from_object)

        (where_clause) @where

        (order_by_clause) @order_by

        (limit_clause) @limit

        (subquery) @subquery
    "#;

    let query = Query::new(language, soql_analysis_query)
        .expect("Failed to create SOQL analysis query");
    let mut cursor = QueryCursor::new();
    cursor.exec(&query, &tree.root_node());

    let mut fields = Vec::new();
    let mut from_objects = Vec::new();
    let mut has_where = false;
    let mut has_order_by = false;
    let mut has_limit = false;
    let mut subquery_count = 0;

    while let Some(captures) = cursor.next_match(&query) {
        for (name, node) in captures {
            match name.as_str() {
                "field" => {
                    fields.push(node.utf8_text(source).to_string());
                }
                "from_object" => {
                    from_objects.push(node.utf8_text(source).to_string());
                }
                "where" => has_where = true,
                "order_by" => has_order_by = true,
                "limit" => has_limit = true,
                "subquery" => subquery_count += 1,
                _ => {}
            }
        }
    }

    if !from_objects.is_empty() {
        println!("  FROM: {}", from_objects.join(", "));
    }

    if !fields.is_empty() {
        println!("  SELECT fields: {} field(s)", fields.len());
        for field in fields.iter().take(5) {
            println!("    - {}", field);
        }
        if fields.len() > 5 {
            println!("    ... and {} more", fields.len() - 5);
        }
    }

    if has_where {
        println!("  ✓ Has WHERE clause");
    }
    if has_order_by {
        println!("  ✓ Has ORDER BY clause");
    }
    if has_limit {
        println!("  ✓ Has LIMIT clause");
    }
    if subquery_count > 0 {
        println!("  ✓ Contains {} subquery(ies)", subquery_count);
    }
}

