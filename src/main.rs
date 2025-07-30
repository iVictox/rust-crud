use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use mysql_async::{Pool, Opts, prelude::*};
use serde::{Serialize, Deserialize};
use dotenv::dotenv;
use std::env;

/// Estructura que representa una entrada de cine en la base de datos.
#[derive(Debug, Serialize, Deserialize, FromRow)]
struct Entrada {
    id: Option<u32>, 
    numero_cedula: String,
    nombre_cliente: String,
    nombre_funcion: String,
    cantidad_entradas: u32,
    horario_funcion: String,
}

/// Estructura para la creación de una nueva entrada.
#[derive(Debug, Serialize, Deserialize)]
struct CrearEntrada {
    numero_cedula: String,
    nombre_cliente: String,
    nombre_funcion: String,
    cantidad_entradas: u32,
    horario_funcion: String,
}

/// Estructura para la actualización de una entrada.
#[derive(Debug, Serialize, Deserialize)]
struct ActualizarEntrada {
    numero_cedula: Option<String>,
    nombre_cliente: Option<String>,
    nombre_funcion: Option<String>,
    cantidad_entradas: Option<u32>,
    horario_funcion: Option<String>,
}

/// Función para obtener la pool de conexiones a la base de datos.
async fn obtener_pool_db() -> Result<Pool, Box<dyn std::error::Error>> {
    dotenv().ok(); 
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL debe estar configurada en el archivo .env");
    let opts = Opts::from_url(&database_url)?;
    Ok(Pool::new(opts))
}

/// Handler para obtener todas las entradas de cine.
async fn obtener_entradas(pool: web::Data<Pool>) -> impl Responder {
    let mut conn = match pool.get_conn().await {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Error al obtener conexión: {:?}", e);
            return HttpResponse::InternalServerError().json("Error al conectar a la base de datos");
        }
    };

    let result = conn.query_map(
        "SELECT id, numero_cedula, nombre_cliente, nombre_funcion, cantidad_entradas, horario_funcion FROM entradas",
        |(id, numero_cedula, nombre_cliente, nombre_funcion, cantidad_entradas, horario_funcion)| {
            Entrada {
                id: Some(id),
                numero_cedula,
                nombre_cliente,
                nombre_funcion,
                cantidad_entradas,
                horario_funcion,
            }
        }
    ).await;

    match result {
        Ok(entradas) => HttpResponse::Ok().json(entradas),
        Err(e) => {
            eprintln!("Error al consultar entradas: {:?}", e);
            HttpResponse::InternalServerError().json("Error al obtener entradas")
        }
    }
}

/// Handler para obtener una entrada específica por su ID.
async fn obtener_entrada_por_id(pool: web::Data<Pool>, path: web::Path<u32>) -> impl Responder {
    let entrada_id = path.into_inner();
    let mut conn = match pool.get_conn().await {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Error al obtener conexión: {:?}", e);
            return HttpResponse::InternalServerError().json("Error al conectar a la base de datos");
        }
    };

    let result = conn.exec_first(
        "SELECT id, numero_cedula, nombre_cliente, nombre_funcion, cantidad_entradas, horario_funcion FROM entradas WHERE id = :id",
        params! { "id" => entrada_id }
    ).await;

    match result {
        Ok(Some((id, numero_cedula, nombre_cliente, nombre_funcion, cantidad_entradas, horario_funcion))) => {
            HttpResponse::Ok().json(Entrada {
                id: Some(id),
                numero_cedula,
                nombre_cliente,
                nombre_funcion,
                cantidad_entradas,
                horario_funcion,
            })
        },
        Ok(None) => HttpResponse::NotFound().json("Entrada no encontrada"),
        Err(e) => {
            eprintln!("Error al consultar entrada: {:?}", e);
            HttpResponse::InternalServerError().json("Error al obtener entrada")
        }
    }
}

/// Handler para crear una nueva entrada de cine.
async fn crear_entrada(pool: web::Data<Pool>, entrada_data: web::Json<CrearEntrada>) -> impl Responder {
    let mut conn = match pool.get_conn().await {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Error al obtener conexión: {:?}", e);
            return HttpResponse::InternalServerError().json("Error al conectar a la base de datos");
        }
    };

    let result = conn.exec_drop(
        "INSERT INTO entradas (numero_cedula, nombre_cliente, nombre_funcion, cantidad_entradas, horario_funcion) VALUES (:numero_cedula, :nombre_cliente, :nombre_funcion, :cantidad_entradas, :horario_funcion)",
        params! {
            "numero_cedula" => &entrada_data.numero_cedula,
            "nombre_cliente" => &entrada_data.nombre_cliente,
            "nombre_funcion" => &entrada_data.nombre_funcion,
            "cantidad_entradas" => entrada_data.cantidad_entradas,
            "horario_funcion" => &entrada_data.horario_funcion,
        }
    ).await;
    
    // Manejo de error específico para cedulas duplicadas
    match result {
        Ok(_) => HttpResponse::Created().json("Entrada creada exitosamente"),
        Err(e) => {
            eprintln!("Error al crear entrada: {:?}", e);
            if e.to_string().contains("Duplicate entry") {
                HttpResponse::Conflict().json("El número de cédula ya existe para otra entrada")
            } else {
                HttpResponse::InternalServerError().json("Error al crear entrada")
            }
        }
    }
}

/// Handler para actualizar una entrada de cine existente.
async fn actualizar_entrada(
    pool: web::Data<Pool>,
    path: web::Path<u32>,
    entrada_data: web::Json<ActualizarEntrada>,
) -> impl Responder {
    let entrada_id = path.into_inner();
    let mut conn = match pool.get_conn().await {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Error al obtener conexión: {:?}", e);
            return HttpResponse::InternalServerError().json("Error al conectar a la base de datos");
        }
    };

    let mut query_parts = Vec::new();

    let mut params_vec = Vec::new(); 
    params_vec.push(("id".to_string(), mysql_async::Value::from(entrada_id)));

    if let Some(numero_cedula) = &entrada_data.numero_cedula {
        query_parts.push("numero_cedula = :numero_cedula".to_string());
        params_vec.push(("numero_cedula".to_string(), numero_cedula.clone().into()));
    }
    if let Some(nombre_cliente) = &entrada_data.nombre_cliente {
        query_parts.push("nombre_cliente = :nombre_cliente".to_string());
        params_vec.push(("nombre_cliente".to_string(), nombre_cliente.clone().into()));
    }
    if let Some(nombre_funcion) = &entrada_data.nombre_funcion {
        query_parts.push("nombre_funcion = :nombre_funcion".to_string());
        params_vec.push(("nombre_funcion".to_string(), nombre_funcion.clone().into()));
    }
    if let Some(cantidad_entradas) = entrada_data.cantidad_entradas {
        query_parts.push("cantidad_entradas = :cantidad_entradas".to_string());
        params_vec.push(("cantidad_entradas".to_string(), cantidad_entradas.into()));
    }
    if let Some(horario_funcion) = &entrada_data.horario_funcion {
        query_parts.push("horario_funcion = :horario_funcion".to_string());
        params_vec.push(("horario_funcion".to_string(), horario_funcion.clone().into()));
    }

    if query_parts.is_empty() {
        return HttpResponse::BadRequest().json("No se proporcionaron datos para actualizar");
    }

    let query = format!("UPDATE entradas SET {} WHERE id = :id", query_parts.join(", "));
    let result = conn.exec_drop(query, params_vec).await;

    match result {
        Ok(_) => {
            let affected_rows = conn.affected_rows();
            if affected_rows == 0 {
                HttpResponse::NotFound().json("Entrada no encontrada o sin cambios")
            } else {
                HttpResponse::Ok().json("Entrada actualizada exitosamente")
            }
        },
        Err(e) => {
            eprintln!("Error al actualizar entrada: {:?}", e);
            if e.to_string().contains("Duplicate entry") {
                HttpResponse::Conflict().json("El número de cédula ya existe para otra entrada")
            } else {
                HttpResponse::InternalServerError().json("Error al actualizar entrada")
            }
        }
    }
}

/// Handler para eliminar una entrada de cine por su ID.
async fn eliminar_entrada(pool: web::Data<Pool>, path: web::Path<u32>) -> impl Responder {
    let entrada_id = path.into_inner();
    let mut conn = match pool.get_conn().await {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Error al obtener conexión: {:?}", e);
            return HttpResponse::InternalServerError().json("Error al conectar a la base de datos");
        }
    };

    let result = conn.exec_drop(
        "DELETE FROM entradas WHERE id = :id",
        params! { "id" => entrada_id }
    ).await;

    match result {
        Ok(_) => {
            let affected_rows = conn.affected_rows();
            if affected_rows == 0 {
                HttpResponse::NotFound().json("Entrada no encontrada")
            } else {
                HttpResponse::Ok().json("Entrada eliminada exitosamente")
            }
        },
        Err(e) => {
            eprintln!("Error al eliminar entrada: {:?}", e);
            HttpResponse::InternalServerError().json("Error al eliminar entrada")
        }
    }
}

/// Función principal 
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = match obtener_pool_db().await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Fallo al inicializar la pool de la base de datos: {:?}", e);
            std::process::exit(1); // Sale si no se puede conectar a la DB
        }
    };

    println!("El servidor ha iniciado en la ruta: http://127.0.0.1:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone())) 
            .service(
                web::scope("/entradas") // Todas las rutas bajo /entradas
                    .route("", web::get().to(obtener_entradas))
                    .route("", web::post().to(crear_entrada))
                    .route("/{id}", web::get().to(obtener_entrada_por_id))
                    .route("/{id}", web::put().to(actualizar_entrada))
                    .route("/{id}", web::delete().to(eliminar_entrada)),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
