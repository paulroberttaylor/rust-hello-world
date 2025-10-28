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
