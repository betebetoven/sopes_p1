
# **Módulo del Kernel para el Monitoreo del Uso de Memoria y CPU en Contenedores**

### **Autor**: Alberto Josué Hernández Armas  
### **Carnet**: 201903553  
### **Versión**: 1.3  
### **Licencia**: GPL

## **Introducción**
Este proyecto consiste en un módulo del kernel de Linux diseñado para monitorear el uso de memoria y CPU de los procesos relacionados con contenedores, específicamente enfocándose en los procesos de `containerd-shim`. El módulo calcula varias métricas, como el tamaño del conjunto residente (RSS), el uso de memoria virtual (VSZ), y los porcentajes de uso de CPU, y luego muestra la información en formato JSON a través del sistema de archivos `/proc`.

El objetivo es recopilar datos precisos similares a los proporcionados por `docker stats`, pero enfocados en atributos específicos como el uso de CPU y memoria sin requerir métricas adicionales o sobrecargar con datos extra.

## **Características**
- **Cálculo del Uso de Memoria**: Calcula el RSS y VSZ (tamaño de memoria virtual) para los procesos relacionados con contenedores.
- **Cálculo del Uso de CPU**: Monitorea el uso de CPU en intervalos de tiempo utilizando jiffies, soportando múltiples núcleos de CPU.
- **Identificación del Contenedor**: Extrae los IDs de los contenedores de los argumentos de la línea de comandos pasados a los procesos de `containerd-shim`.
- **Cálculo Recursivo de Recursos**: Acumula el uso de memoria y CPU para los procesos hijos, proporcionando una vista completa del consumo de recursos de los contenedores.
- **Clasificación de Alto Rendimiento y Bajo Rendimiento**: Basado en los umbrales de uso de CPU y memoria, el módulo clasifica los contenedores como de alto o bajo rendimiento.

## **Estructura de Archivos**

### **Archivos del Módulo del Kernel:**
1. **`container_mem_info_module.c`**  
   Archivo principal del módulo del kernel. Define todas las funciones necesarias para extraer información sobre el uso de memoria y CPU de los contenedores, y proporciona una interfaz a través de `/proc`.

2. **`container_info_201903553` (Entrada en Proc)**  
   Un archivo en el sistema de archivos `/proc` creado por el módulo del kernel. Almacena la información sobre el uso de memoria y CPU de los procesos `containerd-shim` en formato JSON.

### **Scripts Auxiliares:**
1. **`generate_containers.sh`**  
   Un script bash que genera y ejecuta de forma aleatoria 10 contenedores Docker, con diferentes niveles de consumo de recursos (alto CPU, alto RAM, bajo consumo). A cada contenedor se le asigna un nombre aleatorio utilizando `/dev/urandom`.

### **Utilidad en Rust para el Análisis**:  
Este programa en Rust analiza la salida en JSON de `/proc/container_info_201903553` para clasificar los contenedores en función de su rendimiento, proporcionando un informe detallado de contenedores de alto rendimiento y bajo rendimiento.

## **Instalación**

### **Requisitos**:
1. **Kernel de Linux 5.x o superior**
2. **Docker** (para pruebas con contenedores)
3. **Rust** (para analizar la salida)

### **Pasos para Instalar el Módulo del Kernel**:

1. **Compilar el Módulo del Kernel**:
   - Navegar al directorio que contiene el archivo `container_mem_info_module.c`.
   - Ejecutar el siguiente comando para compilar el módulo del kernel:
     ```bash
     make
     ```

2. **Cargar el Módulo del Kernel**:
   - Una vez compilado el módulo, cargarlo en el kernel utilizando `insmod`:
     ```bash
     sudo insmod container_mem_info_module.ko
     ```

3. **Verificar la Entrada en `/proc`**:
   - Verificar que se haya creado el archivo `/proc/container_info_201903553`:
     ```bash
     cat /proc/container_info_201903553
     ```

4. **Monitorear la Salida**:
   - El archivo `/proc/container_info_201903553` contendrá datos en formato JSON con la información de memoria y uso de CPU de los procesos de `containerd-shim`.

5. **Descargar el Módulo del Kernel**:
   - Para eliminar el módulo del kernel:
     ```bash
     sudo rmmod container_mem_info_module
     ```

## **Uso**

1. **Monitoreo del Rendimiento de los Contenedores**:
   - Después de cargar el módulo del kernel, utilizar el siguiente comando para leer la información de memoria y CPU de los contenedores:
     ```bash
     cat /proc/container_info_201903553
     ```

2. **Clasificación de Contenedores**:
   - La utilidad en Rust analiza la salida JSON de `/proc/container_info_201903553` para clasificar los contenedores según su rendimiento:
     ```bash
     cargo run --release
     ```

3. **Ejecución de Contenedores Docker**:
   - El script bash `generate_containers.sh` crea de manera aleatoria contenedores Docker simulando cargas de trabajo de alto y bajo consumo:
     ```bash
     ./generate_containers.sh
     ```

## **Clasificación de Contenedores de Alto Rendimiento y Bajo Rendimiento**

La clasificación se basa en los siguientes criterios:

### **Contenedores de Bajo Rendimiento**:
- Uso de CPU ≤ 0.09%
- Uso de memoria ≤ 0.16% de la memoria total del sistema

### **Contenedores de Alto Rendimiento**:
- Uso de CPU > 0.09%
- Uso de memoria > 0.16% de la memoria total del sistema

Los datos en formato JSON de `/proc/container_info_201903553` son analizados para clasificar los contenedores.

## **Formato de Datos (JSON)**
```json
{
  "total_memory_kb": "8235000",
  "free_memory_kb": "512000",
  "used_memory_kb": "7728000",
  "processes": [
    {
      "process_name": "containerd-shim",
      "pid": "12345",
      "container_id": "abc123",
      "vsz_kb": "1248576",
      "rss_kb": "14336",
      "memory_usage_percent": "0.17",
      "cpu_usage_percent": "0.05"
    },
    ...
  ]
}
```

## **Ejemplo de Salida del Programa en Rust**
- **Contenedores de Bajo Rendimiento**:
  ```
  Contenedores de Bajo Rendimiento:
  PID: 12345
  Nombre: containerd-shim
  ID de Contenedor: abc123
  Uso de Memoria: 0.15% del sistema
  Uso de CPU: 0.02%
  ```

- **Contenedores de Alto Rendimiento**:
  ```
  Contenedores de Alto Rendimiento:
  PID: 67890
  Nombre: containerd-shim
  ID de Contenedor: xyz789
  Uso de Memoria: 1.56% del sistema
  Uso de CPU: 15.34%
  ```

## **Limitaciones Conocidas**
- **Memoria Compartida y Caché**: El módulo calcula el uso de memoria basado en el RSS, lo que no tiene en cuenta la memoria utilizada por los recursos compartidos de Docker (como la memoria en caché).
- **Precisión del Uso de CPU**: El uso de CPU se calcula basándose en `jiffies` y es de tipo instantáneo. Para un monitoreo más detallado, el módulo necesitaría realizar un seguimiento a lo largo de intervalos más largos.

## **Mejoras Futuras**
- **Integración con `cgroups`**: Para un seguimiento más preciso de la memoria y el CPU, la integración con cgroups podría proporcionar estadísticas más detalladas sobre el uso de la memoria y el caché.
- **Monitoreo de Red y E/S**: Las futuras versiones podrían agregar el seguimiento del uso de red y bloques de E/S.
