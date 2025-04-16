package com.example.zenohdemo

import android.os.Bundle
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
import com.example.zenohdemo.databinding.ActivityMainBinding
import io.zenoh.Config
import io.zenoh.Session
import io.zenoh.Zenoh

class MainActivity : AppCompatActivity() {

    private lateinit var appBarConfiguration: AppBarConfiguration
    private lateinit var binding: ActivityMainBinding
    private lateinit var session: Session
    private lateinit var viewModel: ZenohViewModel
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        viewModel = ViewModelProvider(this)[ZenohViewModel::class.java]

        Zenoh.initLogFromEnvOr("error")

        Zenoh.open(Config.default()).onSuccess {
            viewModel.zenohSession = it
        }.onFailure {
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
                R.id.z_pub,
                R.id.z_sub,
                R.id.z_queryable,
                R.id.z_get,
                R.id.z_get_liveliness,
                R.id.z_put,
                R.id.z_delete,
                R.id.z_info,
                R.id.z_liveliness,
                R.id.z_sub_liveliness,
                R.id.z_ping,
                R.id.z_pong,
                R.id.z_pub_thr,
                R.id.z_sub_thr,
                R.id.z_scout
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