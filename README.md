## Lake Cache 
#### A performant, scalable, and durable RESTful key-value store.  
Designed with simplicity in mind for demanding cloud-native and serverless applications.       
  
<img src="https://github.com/a-agmon/lake-cache/blob/main/arch2.png?raw=true" alt="LakeCache" width="100%">

## Introduction
Lake-Cache is a fast and durable HTTP-based key-value store designed for cloud-native and serverless applications. It uses RESTful endpoint instances to handle cache operations and AWS S3 for durable storage. Each endpoint manages its own independent cache, providing isolation and scalability. Keys are persisted to durable storage and then cached for retrieval operations. Cache endpoints sync with durable storage on a configurable item TTL.

## Features
- **RESTful API**: Simple, lean, and intuitive RESTful API for easy integration.
- **Durable Storage**: Uses AWS S3 for durable storage.
- **Isolation**: Each endpoint manages its own independent cache, providing isolation and scalability.
- **Scalability**: Scalable and can handle large number of clients. More cache endpoints can be added to handle more traffic.

