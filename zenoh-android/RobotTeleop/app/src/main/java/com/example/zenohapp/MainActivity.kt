package com.example.zenohapp

import android.os.Bundle
import android.util.Log
import android.view.Menu
import androidx.appcompat.app.AlertDialog
import com.google.android.material.navigation.NavigationView
import androidx.navigation.findNavController
import androidx.navigation.ui.AppBarConfiguration
import androidx.navigation.ui.navigateUp
import androidx.navigation.ui.setupActionBarWithNavController
import androidx.navigation.ui.setupWithNavController
import androidx.drawerlayout.widget.DrawerLayout
import androidx.appcompat.app.AppCompatActivity
import androidx.lifecycle.ViewModelProvider
import com.example.zenohapp.databinding.ActivityMainBinding
import io.zenoh.Config
import io.zenoh.Session
import io.zenoh.Zenoh
import java.io.File
import java.io.FileOutputStream
import kotlin.io.path.Path

class MainActivity : AppCompatActivity() {

    private lateinit var appBarConfiguration: AppBarConfiguration
    private lateinit var binding: ActivityMainBinding
    private lateinit var session: Session
    private lateinit var viewModel: ZenohViewModel
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        viewModel = ViewModelProvider(this).get(ZenohViewModel::class.java)

        Zenoh.initLogFromEnvOr("debug")

        val configFile = assets.open("config-inline-prod.json")
        val tempConfig = File.createTempFile("config", ".json5")
        tempConfig.deleteOnExit()
        FileOutputStream(tempConfig).use { output ->
            configFile.copyTo(output)
        }

        val config = Config.fromFile(Path(tempConfig.absolutePath)).getOrThrow()
        Zenoh.open(config).onSuccess {
            viewModel.zenohSession = it
        }.onFailure {
            Log.e("Zenoh Session", "Zenoh session could not be opened: ${it.message}")
            val alertDialogBuilder = AlertDialog.Builder(this)
            alertDialogBuilder
                .setTitle("Error")
                .setMessage("Zenoh session could not be opened: ${it.message}")
                .setPositiveButton("OK") { dialog, _ ->
                    dialog.dismiss()
                }
                .create()
                .show()
        }

        binding = ActivityMainBinding.inflate(layoutInflater)
        setContentView(binding.root)

        setSupportActionBar(binding.appBarMain.toolbar)

        val drawerLayout: DrawerLayout = binding.drawerLayout
        val navView: NavigationView = binding.navView
        val navController = findNavController(R.id.nav_host_fragment_content_main)
        // Passing each menu ID as a set of Ids because each
        // menu should be considered as top level destinations.
        appBarConfiguration = AppBarConfiguration(
            setOf(
                R.id.z_teleop,
            ), drawerLayout
        )
        setupActionBarWithNavController(navController, appBarConfiguration)
        navView.setupWithNavController(navController)
    }

    override fun onCreateOptionsMenu(menu: Menu): Boolean {
        // Inflate the menu; this adds items to the action bar if it is present.
        menuInflater.inflate(R.menu.main, menu)
        return true
    }

    override fun onSupportNavigateUp(): Boolean {
        val navController = findNavController(R.id.nav_host_fragment_content_main)
        return navController.navigateUp(appBarConfiguration) || super.onSupportNavigateUp()
    }

    override fun onDestroy() {
        super.onDestroy()
        session.close()
    }
}