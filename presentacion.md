# Sistema de Red Social Distribuid# DDBP
---
## Una Arquitectura Distribuida Moderna

- **Proyecto**: Plataforma de Red Social Distribuida
- **Stack Tecnológico**:
  - MongoDB ReplicaSet para almacenamiento de datos distribuido
  - Rust con Actix Web para backend de alto rendimiento
  - Next.js para interfaz frontend moderna
  - Podman/Podman Compose para contenerización

---

# Arquitectura del Sistema

```
┌─────────────────────────┐
│    Frontend (Next.js)   │
├─────────────────────────┤
│  API Gateway (Actix)    │
├─────────────────────────┤
│    Backend (Rust)       │
├─────────────────────────┤
│  MongoDB ReplicaSet     │
├─────────────────────────┤
│ Podman Infrastructure   │
└─────────────────────────┘
```

- **Arquitectura por Capas**:
  - Capa Frontend: Next.js para interfaz de usuario responsiva
  - API Gateway: Actix Web para manejo eficiente de peticiones
  - Backend: Rust para lógica de negocio de alto rendimiento
  - Base de Datos: MongoDB ReplicaSet para almacenamiento distribuido
  - Infraestructura: Podman para orquestación de contenedores

---

# Arquitectura de Red

```
   Client App
        ↓
    API Gateway
        ↓
┌─────────────────┐
│  Primary Node   │
│  (Priority 10)  │
└────┬─────┬─────┘
     ↓     ↓
┌────┴─┐ ┌─┴────┐
│ Sec1 │ │ Sec2 │
│ (P5) │ │ (P1) │
└──────┘ └──────┘
```

- **Configuración del ReplicaSet**:
  - Nodo Primario (Prioridad 10): Operaciones principales de escritura
  - Secundario-1 (Prioridad 5): Primer objetivo de conmutación por error
  - Secundario-2 (Prioridad 1): Redundancia adicional
- Aislamiento de red vía `ddbp_mongo-network`
- Mecanismo de conmutación por error automatizado
- Autenticación mediante keyfile entre nodos

---

# Detalles del Stack Tecnológico

- **Configuración Distribuida de MongoDB**:
  - Configuración ReplicaSet para redundancia
  - Confirmación de escritura por mayoría para durabilidad de datos
  - Autenticación basada en keyfile
  - Connection pooling (5-20 conexiones)

- **Implementación en Rust**:
  - Framework Actix Web para API asíncrona
  - Runtime Tokio para operaciones asíncronas
  - Serde para serialización eficiente
  - Sistema robusto de manejo de errores

- **Características del Sistema**:
  - Timeouts y reintentos configurables
  - Monitoreo de heartbeat (intervalos de 15s)
  - Manejo automático de failover
  - Balanceo de carga entre secundarios

---

# Modelos de Base de Datos

El sistema utiliza los siguientes modelos para almacenar la información en MongoDB:

## User
- **username**: String - Nombre de usuario único
- **email**: String - Correo electrónico del usuario
- **password_hash**: String - Hash de la contraseña para seguridad
- **bio**: Option<String> - Biografía opcional del usuario
- **profile_picture_url**: Option<String> - URL opcional de la imagen de perfil
- **join_date**: Option<String> - Fecha de registro en la plataforma
---
## PostType 
Enumeración (Enum) que define los tipos de publicaciones soportados:
- **Text**: Publicaciones de solo texto
- **Image**: Publicaciones con imágenes
- **Video**: Publicaciones con videos 
- **Link**: Publicaciones con enlaces

## Post 
- **user_id**: String - Identificador del usuario que realiza la publicación
- **content**: String - Contenido textual de la publicación
- **media_urls**: Vec<String> - Lista de URLs a contenido multimedia
- **post_type**: PostType - Tipo de publicación (Text, Image, Video, Link)
- **like_count**: i32 - Contador de "me gusta"
---
## Comment 
- **post_id**: String - Identificador de la publicación comentada
- **user_id**: String - Identificador del usuario que comenta
- **content**: String - Contenido del comentario

## Follow
- **follower_id**: String - Identificador del usuario seguidor
- **following_id**: String - Identificador del usuario seguido

## Like 
- **post_id**: String - Identificador de la publicación
- **user_id**: String - Identificador del usuario que da "me gusta"
- **created_at**: Option<String> - Fecha y hora de la acción
---
## UserProfile
- **id**: String - Identificador único del usuario
- **username**: String - Nombre de usuario
- **bio**: Option<String> - Biografía opcional
- **profile_picture_url**: Option<String> - URL opcional de imagen de perfil
- **join_date**: String - Fecha de registro
- **follower_count**: i32 - Número de seguidores
- **following_count**: i32 - Número de usuarios seguidos
- **post_count**: i32 - Número de publicaciones realizadas
---
## PostDetails
- **id**: String - Identificador único de la publicación
- **user_id**: String - Identificador del autor
- **username**: String - Nombre de usuario del autor
- **profile_picture_url**: Option<String> - URL de imagen de perfil del autor
- **content**: String - Contenido de la publicación
- **media_urls**: Vec<String> - URLs de contenido multimedia
- **post_type**: PostType - Tipo de publicación
- **created_at**: String - Fecha de creación en formato ISO
- **human_time**: String - Fecha legible para humanos
- **like_count**: i32 - Número de "me gusta"
- **comment_count**: i32 - Número de comentarios
- **has_liked**: bool - Indica si el usuario actual ha dado "me gusta"
---
## UserStats
- **post_count**: i32 - Número de publicaciones
- **comment_count**: i32 - Número de comentarios
- **follower_count**: i32 - Número de seguidores
- **following_count**: i32 - Número de usuarios seguidos
- **total_likes_received**: i32 - Total de "me gusta" recibidos
- **total_likes_given**: i32 - Total de "me gusta" otorgados

---

# Características y Beneficios Clave

- **Alta Disponibilidad**:
  - Mecanismo de failover automatizado
  - Sin puntos únicos de fallo
  - Almacenamiento redundante de datos

- **Consistencia de Datos**:
  - Confirmaciones de lectura/escritura por mayoría
  - Soporte para operaciones atómicas
  - Modelo de consistencia fuerte
---
- **Resiliencia del Sistema**:
  - Manejo robusto de errores
  - Recuperación automática
  - Monitoreo integral

- **Escalabilidad**:
  - Despliegue containerizado
  - Preparado para escalado horizontal
  - Utilización eficiente de recursos
