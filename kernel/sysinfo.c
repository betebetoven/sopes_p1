#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/init.h>
#include <linux/proc_fs.h>
#include <linux/seq_file.h>
#include <linux/mm.h>
#include <linux/sched.h>
#include <linux/sched/signal.h>
#include <linux/uaccess.h> // For command-line arguments
#include <linux/timer.h>
#include <linux/jiffies.h>
#include <linux/signal.h>

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Your Name");
MODULE_DESCRIPTION("Kernel module to display memory and container process info");
MODULE_VERSION("1.0");

#define PROC_NAME "container_meminfo"
#define TARGET_PROCESS "containerd-shim"

static unsigned long calculate_mem_usage(struct task_struct *task) {
    struct mm_struct *mm = task->mm;
    if (!mm)
        return 0;

    unsigned long rss = get_mm_rss(mm); // RSS in pages
    return rss << (PAGE_SHIFT - 10); // Convert pages to KB
}

static int container_meminfo_show(struct seq_file *m, void *v) {
    struct sysinfo si;
    struct task_struct *task;
    int first_process = 1; // Track the first process for proper comma placement

    // Get memory information
    si_meminfo(&si);
    unsigned long total_ram = si.totalram * 4; // KB
    unsigned long free_ram = si.freeram * 4;   // KB
    unsigned long used_ram = total_ram - free_ram;

    // Start JSON object
    seq_printf(m, "{\n");
    
    // Print system memory info as JSON
    seq_printf(m, "  \"system_memory\": {\n");
    seq_printf(m, "    \"total_ram\": %lu,\n", total_ram);
    seq_printf(m, "    \"free_ram\": %lu,\n", free_ram);
    seq_printf(m, "    \"used_ram\": %lu\n", used_ram);
    seq_printf(m, "  },\n");

    // Print container processes as JSON array
    seq_printf(m, "  \"container_processes\": [\n");

    // Iterate over each task (process)
    for_each_process(task) {
        char comm[TASK_COMM_LEN];
        get_task_comm(comm, task);

        // Only include processes named "containerd-shim"
        if (strcmp(comm, TARGET_PROCESS) == 0) {
            if (task->mm) { // Only consider tasks with memory maps
                unsigned long vsz = task->mm->total_vm << (PAGE_SHIFT - 10); // Vsz in KB
                unsigned long rss = calculate_mem_usage(task);               // RSS in KB
                unsigned long mem_usage_percent = (rss * 100) / total_ram;   // Memory usage percentage

                // Placeholder for CPU usage
                unsigned long total_cpu_usage = task->se.sum_exec_runtime; // CPU usage in nanoseconds

                // Print container process info
                if (!first_process) {
                    seq_printf(m, ",\n"); // Add a comma between process entries
                }
                first_process = 0;

                seq_printf(m, "    {\n");
                seq_printf(m, "      \"pid\": %d,\n", task->pid);
                seq_printf(m, "      \"name\": \"%s\",\n", comm);
                seq_printf(m, "      \"vsz\": %lu,\n", vsz);
                seq_printf(m, "      \"rss\": %lu,\n", rss);
                seq_printf(m, "      \"memory_usage\": %lu,\n", mem_usage_percent);
                seq_printf(m, "      \"cpu_usage\": %lu\n", total_cpu_usage);
                seq_printf(m, "    }");
            }
        }
    }

    // Close JSON array and object
    seq_printf(m, "\n  ]\n");
    seq_printf(m, "}\n");

    return 0;
}

static int container_meminfo_open(struct inode *inode, struct file *file) {
    return single_open(file, container_meminfo_show, NULL);
}

static const struct proc_ops container_meminfo_ops = {
    .proc_open = container_meminfo_open,
    .proc_read = seq_read,
};

static int __init container_meminfo_init(void) {
    proc_create(PROC_NAME, 0, NULL, &container_meminfo_ops);
    printk(KERN_INFO "container_meminfo module loaded\n");
    return 0;
}

static void __exit container_meminfo_exit(void) {
    remove_proc_entry(PROC_NAME, NULL);
    printk(KERN_INFO "container_meminfo module unloaded\n");
}

module_init(container_meminfo_init);
module_exit(container_meminfo_exit);
