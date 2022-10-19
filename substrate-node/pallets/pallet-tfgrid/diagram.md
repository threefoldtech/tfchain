```mermaid
erDiagram
    Entity ||--o{ Twin : optionaly_has_multiple
    Entity {
        int version
        int id
        string name
        int country_id
        int city_id
        string address
    }
    Twin ||--|{ Farm : can_be_a
    Twin ||--|{ Node : can_be_a
    Twin {
        int version
        int id
        string address
        string ip
        list EntityProofs
    }
    Twin ||--|{ Proof : can_have_multiple
    Entity ||--|{ Proof : can_have_multiple
    Proof {
        int entityID
        int twinID
        string signature
    }
    Node ||--|{ Farm : linked_to
    Farm {
        int version
        int id
        string name
        int twin_id
        list public_ips
        string certificationType
        string location
    }
    Farm ||--|{ PricingPolicy : can_have
    Node {
        int version
        int id
        int farm_id
        int twin_id
        role role
        string address
        string role
        string location
    }
    PricingPolicy {
        string name
        int su
        int cu
        int nu
        int ipv4u
    }
```