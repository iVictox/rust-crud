CREATE TABLE entradas (
    id INT AUTO_INCREMENT PRIMARY KEY,
    numero_cedula VARCHAR(255) NOT NULL UNIQUE,
    nombre_cliente VARCHAR(255) NOT NULL,
    nombre_funcion VARCHAR(255) NOT NULL,
    cantidad_entradas INT NOT NULL,
    horario_funcion VARCHAR(255) NOT NULL
);