    package tech.zettascale.demo.location

    import android.Manifest
    import android.annotation.SuppressLint
    import android.content.Context
    import android.content.pm.PackageManager
    import android.os.Bundle
    import androidx.activity.ComponentActivity
    import androidx.activity.compose.setContent
    import androidx.activity.enableEdgeToEdge
    import androidx.compose.foundation.layout.fillMaxSize
    import androidx.compose.foundation.layout.padding
    import androidx.compose.material3.Scaffold
    import androidx.compose.material3.Text
    import androidx.compose.runtime.Composable
    import androidx.compose.ui.Modifier
    import androidx.core.app.ActivityCompat
    import androidx.core.content.ContextCompat
    import tech.zettascale.demo.location.ui.theme.LocationTrackerTheme
    import android.location.Location
    import android.location.LocationListener
    import android.location.LocationManager
    import android.os.Looper
    import android.util.Log
    import android.view.WindowManager
    import androidx.compose.foundation.layout.Arrangement
    import androidx.compose.foundation.layout.Box
    import androidx.compose.foundation.layout.Column
    import androidx.compose.foundation.layout.Row
    import androidx.compose.material3.Button
    import androidx.compose.material3.ButtonColors
    import androidx.compose.material3.TextField
    import androidx.compose.runtime.getValue
    import androidx.compose.runtime.mutableStateOf
    import androidx.compose.runtime.remember
    import androidx.compose.runtime.setValue
    import androidx.compose.ui.Alignment
    import androidx.compose.ui.graphics.Color
    import androidx.compose.ui.unit.dp
    import io.zenoh.Session
    import io.zenoh.Config
    import io.zenoh.keyexpr.intoKeyExpr
    import io.zenoh.prelude.*
    import io.zenoh.publication.Publisher
    import io.zenoh.value.Value


    object LocatorState {
        var lm: LocationManager? = null
        private var locationListener: LocationListener? = null
        private var z: Session? = null
        private var zPub: Publisher? = null
        private var activity: MainActivity? = null
        private var running: Boolean = false
        private var last_locator: String? = null
        private var last_kind: String? = null
        private const val UPDATE_PERIOD: Long = 1000

        fun setupActivity(a: MainActivity) {
            activity = a
        }
        private fun setupZenoh(locator: String, kind: String) {
            if (z != null && locator == last_locator && kind == last_kind) {
                Log.d("ZLoc", "Zenoh Runtime already setup for $locator and $kind")
                return
            } else {
                last_locator = locator
                last_kind = kind
                // Close existing session and publisher
                Log.d("ZLoc", "Cleaning up former zenoh Runtime")
                if (zPub != null) {
                    zPub!!.close()
                    z!!.close()
                }
            }
            Log.d("ZLoc", "Setting up zenoh Runtime")
            val configJSON = """
             {
                 "mode": "client",
                 "connect": {
                     "endpoints": [
                         "$locator"
                     ]
                 },
                 "scouting": {
                     "multicast": {
                         "enabled": false
                     }
                 }
             }
            """.trimIndent()

            val kexpr = "demo/tracker/mobs/$kind".intoKeyExpr().getOrThrow()
            val c = Config.from(configJSON)

            z = Session.open(c).getOrThrow()
            zPub = z!!.declarePublisher(kexpr)
                .congestionControl(CongestionControl.DROP)
                .priority(Priority.REALTIME)
                .res().getOrThrow()
        }

        @SuppressLint("MissingPermission")
        fun startLocationTracking(locator: String,  model: String, color: String,  kind: String) {
            running = true
            Log.d("ZLoc", "Starting/Restarting Location Tracking")

            setupZenoh(locator, kind)

            Log.d("ZLoc", ">>>>>>>> Getting Location Manager")
            lm = activity!!.getLocationManager()
            val provider = lm!!.allProviders.find { s -> s == "fused" } ?: "gps"
            Log.d("ZLoc", ">>>>>>>> Provider: $provider")

            locationListener =
                LocationListener { l ->
                    val value = Value(locationToCarData(l, model, color, kind), Encoding(KnownEncoding.APP_JSON, ""))
                    zPub!!.put(value).res()
                    Log.d("ZLoc", ">>>>>>>> Published Location: $l")
                }

            val p: String = provider
            val t: Long = UPDATE_PERIOD
            val d = 0f

            Log.d("ZLoc", ">>>>>>>> Registering Location Listener")
            lm?.requestLocationUpdates(p, t, d, locationListener!!, Looper.getMainLooper())
        }

        fun stopLocationTracking() {
            Log.d("ZLoc", "Stopping Location Tracking")
            running = false
            if (lm != null && locationListener != null) {
                lm?.removeUpdates(locationListener!!)
            }
        }
        private fun locationToCarData(l: Location, model: String, color: String, kind: String): String =
            // TODO: get kind from UI
            """{"position": {"lat": ${l.latitude},"lng": ${l.longitude}, "speed": ${l.speed*3.6},"color": "$color","id": "$model", "kind": "$kind"}"""


    }
    class MainActivity : ComponentActivity() {
        companion object {
            private const val LOCATION_PERMISSION_REQUEST_CODE = 1
        }

        fun getLocationManager(): LocationManager {
            if (!isLocationPermissionGranted()) {
                requestLocationPermission()
            }
            return getSystemService(Context.LOCATION_SERVICE) as LocationManager
        }
        override fun onDestroy() {
            super.onDestroy()
            Log.d("ZLoc", "Destroy Activity")
        }
        @SuppressLint("MissingPermission")
        override fun onCreate(savedInstanceState: Bundle?) {
            super.onCreate(savedInstanceState)
            LocatorState.setupActivity(this)

            Log.d("ZLoc", "Starting Activity")
            enableEdgeToEdge()
            setContent {
                LocationTrackerTheme {
                    Scaffold(modifier = Modifier.fillMaxSize()) { innerPadding ->
                        Tracker(
                            modifier = Modifier.padding(innerPadding)
                        )
                    }
                }
            }
            window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        }
        private fun isLocationPermissionGranted(): Boolean {
            val perm = ContextCompat.checkSelfPermission(this, Manifest.permission.ACCESS_FINE_LOCATION)
            return  perm == PackageManager.PERMISSION_GRANTED
        }

        private fun requestLocationPermission() {
            ActivityCompat.requestPermissions(
                this,
                arrayOf(
                    Manifest.permission.ACCESS_FINE_LOCATION,
                    Manifest.permission.ACCESS_COARSE_LOCATION),
                LOCATION_PERMISSION_REQUEST_CODE)
        }



    }

    @Composable
    fun Tracker(modifier: Modifier = Modifier) {
        var locator by remember { mutableStateOf("tcp/3.71.106.121:7447") }
        var model by remember { mutableStateOf("Scrambler 1100") }
        var color by remember { mutableStateOf("Black") }
        var running by remember {mutableStateOf(LocatorState.lm != null)}
        var kind by remember {mutableStateOf("motorbike")}
        Box(modifier = modifier,
            contentAlignment = Alignment.Center) {

            Column(
                verticalArrangement = Arrangement.Center,
                horizontalAlignment = Alignment.CenterHorizontally
                ) {
                Row(
                    Modifier.padding(20.dp),
                    horizontalArrangement = Arrangement.Center,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text("PaaS:")
                    TextField(
                        value = locator,
                        onValueChange = { s -> locator = s}, enabled = !running)
                }
                Row(
                    Modifier.padding(20.dp),
                    horizontalArrangement = Arrangement.Center,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text("Model:")
                    TextField(value = model, onValueChange = { s -> model = s}, enabled = !running)
                }
                Row(
                    Modifier.padding(20.dp),
                    horizontalArrangement = Arrangement.Center,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text("Color:")
                    TextField(value = color, onValueChange = { s -> color = s}, enabled = !running)
                }
                Row(
                    Modifier.padding(20.dp),
                    horizontalArrangement = Arrangement.Center,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text("kind:")
                    TextField(value = kind, onValueChange = { s -> kind = s}, enabled = !running)
                }
                Row(
                    horizontalArrangement = Arrangement.Center,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Button(
                        enabled = !running,
                        colors = ButtonColors(Color.Green, Color.Black, Color.Gray, Color.Black),
                        onClick = {
                            running = true
                            LocatorState.startLocationTracking(locator, model, color, kind)
                        })
                    {
                        Text("Start")
                    }
                    Button(enabled = running,
                        colors = ButtonColors(Color.Red, Color.Black, Color.Gray, Color.Black),
                        onClick = {
                            running = false
                            LocatorState.stopLocationTracking()
                        }
                    ) {
                        Text("Stop")
                    }
                }
            }
        }
    }

