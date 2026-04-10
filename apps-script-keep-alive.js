/**
 * ZeroClaw Render Wake-Up Script
 * 
 * Mantiene tu servicio de Render activo evitando que entre en modo "sleep"
 * después de 15 minutos de inactividad.
 * 
 * Instrucciones:
 * 1. Ve a https://script.google.com
 * 2. Crea un nuevo proyecto
 * 3. Copia este código
 * 4. Configura la URL de tu servicio en la variable 'baseUrl'
 * 5. Configura el trigger para ejecutar cada 10 minutos
 */

function mantenerRenderActivo() {
  // ⚠️ CAMBIA ESTA URL por la URL de tu servicio en Render
  // Ejemplo: "https://zeroclaw-abc123.onrender.com"
  var baseUrl = "https://TU-SERVICIO.onrender.com";
  
  // No necesitas cambiar nada más debajo
  var endpoints = [
    "/",
    "/health",
    "/healthz"
  ];

  var opciones = {
    method: "get",
    muteHttpExceptions: true,
    followRedirects: true,
    headers: {
      "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64)",
      "Accept": "text/html,application/json,*/*",
      "Connection": "keep-alive"
    }
  };

  var totalPings = 3;
  var delayEntrePings = 120000; // 2 minutos

  try {
    Logger.log("=== Iniciando ciclo de ping ===");
    
    for (var i = 0; i < totalPings; i++) {
      var endpoint = endpoints[Math.floor(Math.random() * endpoints.length)];
      var url = baseUrl + endpoint;

      var inicio = new Date().getTime();
      var response = UrlFetchApp.fetch(url, opciones);
      var tiempo = new Date().getTime() - inicio;

      Logger.log("Ping " + (i + 1) + " - Endpoint: " + endpoint + " - Status: " + response.getResponseCode() + " - Tiempo: " + tiempo + "ms");

      // Si el servidor estaba dormido, hacer ping extra de refuerzo
      if (tiempo > 5000) {
        Logger.log("Servidor despertando, reforzando...");
        UrlFetchApp.fetch(baseUrl + "/", opciones);
      }

      // Espera entre pings (excepto el último)
      if (i < totalPings - 1) {
        Utilities.sleep(delayEntrePings);
      }
    }
    
    Logger.log("=== Ciclo completado ===");

  } catch (e) {
    Logger.log("Error: " + e.toString());
  }
}

/**
 * Función alternativa más simple para usar con trigger
 * Ejecuta un solo ping cada 10 minutos
 */
function simplePing() {
  var baseUrl = "https://TU-SERVICIO.onrender.com";
  
  try {
    var response = UrlFetchApp.fetch(baseUrl + "/health", {
      method: "get",
      muteHttpExceptions: true
    });
    
    Logger.log("Ping: " + response.getResponseCode() + " - " + new Date());
  } catch (e) {
    Logger.log("Error: " + e.toString());
  }
}