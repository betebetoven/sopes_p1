// Module for collecting container process memory and CPU usage information
#include <linux/uaccess.h>
#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/init.h>
#include <linux/proc_fs.h>
#include <linux/seq_file.h>
#include <linux/mm.h>
#include <linux/sched.h>
#include <linux/timer.h>
#include <linux/jiffies.h>

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Alberto Josué Hernández Armas");
MODULE_DESCRIPTION("Kernel module to monitor container process memory and CPU usage");
MODULE_VERSION("1.3");

#define PROC_ENTRY_NAME "container_info_201903553"
#define TARGET_PROC_NAME "containerd-shim"

// Prototype for function to calculate RSS (Resident Set Size)
unsigned long calculate_memory_usage(struct task_struct *task);

// Function to extract container ID from command line arguments
char* extract_container_id(struct task_struct *task) {
    unsigned long arg_start = task->mm->arg_start;
    unsigned long arg_end = task->mm->arg_end;
    int len = arg_end - arg_start;

    if (len > 0 && len < 350) {
        static char cmdlineBuffer[350];
        memset(cmdlineBuffer, 0, sizeof(cmdlineBuffer));
        int read = access_process_vm(task, arg_start, cmdlineBuffer, len, FOLL_FORCE);
        if (read > 0) {
            // Replace null characters with spaces for easier parsing
            for (int i = 0; i < read - 1; i++) {
                if (cmdlineBuffer[i] == '\0') {
                    cmdlineBuffer[i] = ' ';
                }
            }
            cmdlineBuffer[read] = '\0';  // Properly terminate the string

            // Find "-id" in the command line
            char *id_pos = strstr(cmdlineBuffer, "-id");
            if (id_pos) {
                id_pos += 4;  // Skip past "-id "
                char *end_id = strchr(id_pos, ' ');
                if (end_id) {
                    *end_id = '\0';  // Terminate the ID at the next space
                }
                return id_pos;
            }
        }
    }
    return "N/A";  // If no ID is found
}

// Recursive function to accumulate resources of all descendant processes
void accumulate_resources(struct task_struct *task, unsigned long *vsz, unsigned long *rss) {
    struct task_struct *child;
    struct list_head *list;

    // Traverse all child processes
    list_for_each(list, &task->children) {
        child = list_entry(list, struct task_struct, sibling);

        // If the child has a memory descriptor, accumulate its memory usage
        if (child->mm) {
            *vsz += child->mm->total_vm * (PAGE_SIZE / 1024);  // Virtual memory size
            *rss += get_mm_rss(child->mm) * (PAGE_SIZE / 1024); // Resident Set Size
        }

        // Recursively accumulate the resources of the child's children (descendants)
        accumulate_resources(child, vsz, rss);
    }
}


// Optimized function to display system information
static int display_container_info(struct seq_file *m, void *v) {
    struct sysinfo si;
    struct task_struct *task;
    int first_process = 1;

    si_meminfo(&si);
    unsigned long total_memory_kb = si.totalram * (PAGE_SIZE / 1024);
    unsigned long free_memory_kb = si.freeram * (PAGE_SIZE / 1024);
    unsigned long used_memory_kb = total_memory_kb - free_memory_kb;

    seq_printf(m, "{\n  \"total_memory_kb\": \"%lu\",\n  \"free_memory_kb\": \"%lu\",\n  \"used_memory_kb\": \"%lu\",\n  \"processes\": [\n",
               total_memory_kb, free_memory_kb, used_memory_kb);

    // Traverse all processes
    for_each_process(task) {
        if (strcmp(task->comm, TARGET_PROC_NAME) != 0) {
            continue;
        }

        if (!first_process) {
            seq_printf(m, "},\n");
        }
        first_process = 0;

        char *container_id = extract_container_id(task);

        seq_printf(m, "   {\n     \"process_name\":\"%s\",\n     \"pid\": \"%d\",\n     \"container_id\": \"%s\",\n",
                   task->comm, task->pid, container_id);

        if (task->mm) {
            unsigned long vsz_kb = task->mm->total_vm * (PAGE_SIZE / 1024);
            unsigned long rss_kb = calculate_memory_usage(task);
            unsigned long memory_usage_percent = (rss_kb * 10000) / total_memory_kb;
            unsigned long total_cpu_time = task->utime + task->stime;
            unsigned long cpu_usage_percent = (total_cpu_time * 10000) / jiffies;

            accumulate_resources(task, &vsz_kb, &rss_kb);

            // Add quotes around the numeric values to store them as strings in JSON
            seq_printf(m, "     \"vsz_kb\":\"%lu\",\n     \"rss_kb\":\"%lu\",\n     \"memory_usage_percent\":\"%lu.%02lu\",\n     \"cpu_usage_percent\":\"%lu.%02lu\"\n",
                       vsz_kb, rss_kb, memory_usage_percent / 100, memory_usage_percent % 100, cpu_usage_percent / 100, cpu_usage_percent % 100);
        } else {
            seq_printf(m, "     \"vsz_kb\": \"0\",\n     \"rss_kb\": \"0\",\n     \"memory_usage_percent\": \"0\",\n     \"cpu_usage_percent\": \"0\"\n");
        }
    }

    if (!first_process) {
        seq_printf(m, "}\n");
    }
    seq_printf(m, "]\n}\n");
    return 0;
}


// Open handler for /proc entry
static int open_container_info(struct inode *inode, struct file *file) {
    return single_open(file, display_container_info, NULL);
}

// Operations struct for /proc entry
static const struct proc_ops container_info_ops = {
    .proc_open = open_container_info,
    .proc_read = seq_read,
};

// Init function to load the module and create the /proc entry
static int __init container_info_init(void) {
    proc_create(PROC_ENTRY_NAME, 0, NULL, &container_info_ops);
    printk(KERN_INFO "Container memory and CPU usage module loaded successfully.\n");
    return 0;
}

// Exit function to clean up when module is unloaded
static void __exit container_info_exit(void) {
    remove_proc_entry(PROC_ENTRY_NAME, NULL);
    printk(KERN_INFO "Container memory and CPU usage module unloaded.\n");
}

// Function to calculate RSS (Resident Set Size) in KB
unsigned long calculate_memory_usage(struct task_struct *task) {
    return task->mm ? get_mm_rss(task->mm) * (PAGE_SIZE / 1024) : 0;
}

module_init(container_info_init);
module_exit(container_info_exit);
