# DDBP - Distributed Social Media Platform

A high-performance, distributed social media platform built with Rust and MongoDB ReplicaSet, featuring modern architecture, robust error handling, and horizontal scalability.

## 🏗️ Architecture Overview

The system implements a distributed architecture using MongoDB ReplicaSet for high availability and data consistency:

### Core Components

- **Backend**: Rust with Actix Web framework
- **Database**: MongoDB ReplicaSet (3 nodes: 1 primary, 2 secondaries)
- **Frontend**: Next.js application
- **Infrastructure**: Podman/Docker containers with container orchestration

### ReplicaSet Configuration

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Primary   │    │ Secondary 1 │    │ Secondary 2 │
│ Priority: 10│◄──►│ Priority: 5 │    │ Priority: 1 │
│  Port: 27017│    │ Port: 27018 │    │ Port: 27019 │
└─────────────┘    └─────────────┘    └─────────────┘
       ▲                  ▲                  ▲
       │                  │                  │
       └──────────────────┼──────────────────┘
                          │
                ┌─────────────┐
                │ Rust App    │
                │ (Actix Web) │
                └─────────────┘
```

## 🚀 Quick Start

### Prerequisites

- Rust (latest stable)
- Node.js 18+
- Podman or Docker
- pnpm

### Installation

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd DDBP
   ```

2. **Set up MongoDB ReplicaSet**
   ```bash
   # Create keyfile for MongoDB authentication
   ./create-keyfile.sh
   
   # Start MongoDB containers
   podman-compose up -d
   ```

3. **Start the Rust backend**
   ```bash
   cd rust-app
   cargo run
   ```

4. **Start the Next.js frontend**
   ```bash
   cd my-app
   pnpm install
   pnpm run dev
   ```

### Using the Scripts

```bash
# Start all services
./start-app.sh

# Stop all services
./stop-app.sh

# Test API endpoints
./api_tests.sh
```

## 📊 Database Schema

### Users Collection
```javascript
{
  "_id": ObjectId(),
  "username": "string",         // Unique, indexed
  "email": "string",           // Unique, indexed
  "password_hash": "string",
  "profile": {
    "full_name": "string",
    "bio": "string",
    "avatar_url": "string",
    "created_at": ISODate(),
    "last_login": ISODate()
  },
  "followers_count": NumberInt,
  "following_count": NumberInt,
  "posts_count": NumberInt
}
```

### Posts Collection
```javascript
{
  "_id": ObjectId(),
  "user_id": ObjectId(),
  "content": "string",
  "media_urls": ["string"],
  "created_at": ISODate(),
  "likes_count": NumberInt,
  "comments_count": NumberInt,
  "tags": ["string"],
  "location": {
    "type": "Point",
    "coordinates": [NumberDouble, NumberDouble]
  }
}
```

## 🔌 API Endpoints

### User Operations
- `POST /create_user` - Create new user account
- `GET /user/{user_id}` - Get user profile
- `PUT /user/{user_id}` - Update user profile

### Social Interactions
- `POST /create_post` - Create new post
- `POST /create_comment` - Add comment to post
- `POST /follow_user` - Follow another user
- `POST /like_post` - Like a post

### System Operations
- `GET /health` - Health check endpoint
- `POST /test/populate` - Populate database with test data
- `POST /test/clean` - Clean test data

## ⚙️ Configuration Features

### High Availability
- **ReplicaSet with 3 nodes** ensuring automatic failover
- **Priority-based election** (10-5-1) for predictable primary selection
- **Read preference**: SecondaryPreferred for load distribution
- **Write concern**: Majority for data durability
- **Read concern**: Majority for consistency

### Connection Pool Settings
```rust
client_options.max_pool_size = Some(20);
client_options.min_pool_size = Some(5);
client_options.max_idle_time = Some(Duration::from_secs(60));
```

### Timeout Configuration
```rust
client_options.connect_timeout = Some(Duration::from_secs(10));
client_options.server_selection_timeout = Some(Duration::from_secs(15));
```

### Retry Policies
```rust
client_options.retry_reads = Some(true);
client_options.retry_writes = Some(true);
```

## 🔒 Security Features

- **Keyfile authentication** for ReplicaSet internal communication
- **Role-based access control (RBAC)** for database operations
- **Network isolation** using dedicated container networks
- **Password hashing** for user authentication
- **Input validation** on all API endpoints

## 📈 Performance Optimizations

### Database Indexing
- Unique indexes on `username` and `email`
- Compound indexes for frequent queries
- Geospatial indexes for location-based features
- Text indexes for content search

### Denormalization Strategy
- Pre-calculated counters (followers, posts, likes)
- Embedded profile information
- Optimized for read-heavy workloads

## 🧪 Testing

### API Testing
```bash
# Run comprehensive API tests
./api_tests.sh
```

### Database Population for Testing
```bash
curl -X POST http://localhost:8080/test/populate \
  -H "Content-Type: application/json" \
  -d '{"users_count": 100, "posts_per_user": 10, "comments_per_post": 5}'
```

### Clean Test Data
```bash
curl -X POST http://localhost:8080/test/clean
```

## 📦 Project Structure

```
DDBP/
├── my-app/                 # Next.js frontend
│   ├── app/               # App router pages
│   ├── components/        # React components
│   └── lib/              # Utility functions
├── rust-app/              # Rust backend
│   ├── src/
│   │   ├── main.rs       # Application entry point
│   │   ├── handlers.rs   # API route handlers
│   │   ├── models.rs     # Data models
│   │   ├── state.rs      # Application state
│   │   └── errors.rs     # Error handling
│   └── Cargo.toml        # Rust dependencies
├── docker-compose.yml     # Container orchestration
├── setup-replica.sh       # MongoDB ReplicaSet setup
├── create-keyfile.sh      # MongoDB authentication setup
├── start-app.sh          # Application startup script
├── stop-app.sh           # Application shutdown script
└── api_tests.sh          # API testing script
```

## 🔧 Development

### Building the Backend
```bash
cd rust-app
cargo build --release
```

### Building the Frontend
```bash
cd my-app
pnpm build
```

### Linting and Formatting
```bash
# Rust
cargo fmt
cargo clippy

# Next.js
pnpm lint
```

## 📊 Monitoring and Health Checks

### Application Health
```bash
curl http://localhost:8080/health
```

### MongoDB ReplicaSet Status
```bash
# Connect to MongoDB shell
mongosh --host localhost:27017 -u admin -p password

# Check replica set status
rs.status()
```

## 🚀 Deployment

### Production Considerations
- Configure proper MongoDB authentication
- Set up SSL/TLS for secure connections
- Implement proper logging and monitoring
- Configure backup strategies
- Set up load balancers for multiple app instances

### Environment Variables
```bash
MONGO_URI=mongodb://admin:password@localhost:27017,localhost:27018,localhost:27019/social_media_db?replicaSet=rs0
RUST_LOG=info
PORT=8080
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is part of an academic assignment for DDBP (Distributed Database Systems) course.

## 👥 Authors

- **Jesus Eduardo Cornejo Clavel** - Student
- **Eduardo Cornejo-Velazquez** - Professor

## 🔗 Related Documentation

- [MongoDB ReplicaSet Documentation](https://docs.mongodb.com/manual/replication/)
- [Actix Web Documentation](https://actix.rs/)
- [Next.js Documentation](https://nextjs.org/docs)
- [Rust Documentation](https://doc.rust-lang.org/)