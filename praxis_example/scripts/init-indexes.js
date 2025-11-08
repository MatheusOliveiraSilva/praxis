db = db.getSiblingDB('praxis');

// Threads collection indexes
db.threads.createIndex({ "user_id": 1, "created_at": -1 });
db.threads.createIndex({ "_id": 1, "user_id": 1 });

// Messages collection indexes
db.messages.createIndex({ "user_id": 1, "thread_id": 1, "created_at": 1 });
db.messages.createIndex({ "thread_id": 1, "created_at": 1 });
db.messages.createIndex({ "user_id": 1, "created_at": -1 });

print("Indexes created successfully");

