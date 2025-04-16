package com.example.zenohdemo.ui.examples.utils

import android.view.LayoutInflater
import android.view.ViewGroup
import androidx.recyclerview.widget.RecyclerView
import com.example.zenohdemo.R

class ConsoleAdapter: RecyclerView.Adapter<ConsoleViewHolder>() {

    private val entries = ArrayList<String>()

    fun addNewEntry(entry: String) {
        entries.add(entry)
        notifyItemInserted(entries.size - 1)
    }

    override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): ConsoleViewHolder {
        val view = LayoutInflater.from(parent.context)
            .inflate(R.layout.view_holder, parent, false)
        return ConsoleViewHolder(view)
    }

    override fun getItemCount(): Int {
        return entries.size
    }


    override fun onBindViewHolder(holder: ConsoleViewHolder, position: Int) {
        holder.entry.text = entries[position]
    }
}

