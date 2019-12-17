use std::{
    env,
    process,
    thread,
    time::{
        Duration,
        SystemTime
    }
};

use tsdb::{
    RRDB,
    Object,
    Parameter,
    ReportId,
};
use sys_info::*;


fn main() {
    let mut num_objects = 1;
    let mut num_parameters = 1;
    let mut id_object = 1;
    let mut id_parameter = 1;
    let mut time_start = 1;
    let mut time_end = unix_time();
    let mut wait_type = 0;
    let mut read_mode = false;

    for arg in env::args() {
        let argument = arg.as_str();
        match argument {
            "-h" => {
                println!("entropy_generator - test programm for make timeseries\n\
                Usage for write: entropy_generator -p 60 -o 50\n\
                where 60 - numeric of parameters, default 1;\n\
                50 - numeric of objects, default 1\n\
                Usage for reading: entropy_generator -read -p 60 -o 50 -pr 2 -po 3 -i 3600 [-end 1200]\n\
                where 60 - numeric of parameters, default 1;\n\
                50 - numeric of objects, default 1\n\
                2 - id of need parameter, default 1\n\
                3 - id of need object, default 1\n\
                3600 - seach interval in seconds\n\
                1200 - optional last seaching time\n");
                process::exit(1);
            },
            "-read" => read_mode = true,
            "-p" => wait_type = 1,
            "-o" => wait_type = 2,
            "-pr" => wait_type = 10,
            "-po" => wait_type = 20,
            "-i" => wait_type = 3,
            "-end" => wait_type = 4,
            _ => {
                match wait_type {
                    1 => num_parameters = argument.parse::<usize>().unwrap_or(0),
                    2 => num_objects = argument.parse::<usize>().unwrap_or(0),
                    3 => time_start = unix_time() - argument.parse::<u64>().unwrap_or(0),
                    4 => time_end = unix_time() - argument.parse::<u64>().unwrap_or(0),
                    10 => id_parameter = argument.parse::<u32>().unwrap_or(0),
                    20 => id_object = argument.parse::<u32>().unwrap_or(0),
                    _ => {},
                };
                wait_type = 0;
            }
        }
    }

    let mut rrdb = RRDB::new("base.rr").unwrap();
    let report_id_vec = make_config(&mut rrdb, num_objects, num_parameters);
    
    if read_mode {
        let report_id = ReportId {
            parameter: id_parameter,
            object: id_object,
        };
        read_data(rrdb, report_id, time_start, time_end);
    } else {
        write_data(rrdb, report_id_vec);
    }
}


fn read_data(mut rrdb: RRDB, report_id: ReportId, time_start: u64, time_end: u64) {
    println!(
        "Read mode start! Time start: {:x}, time end: {:x}, parameter: {:x}, object: {:x}",
        time_start,
        time_end,
        report_id.parameter,
        report_id.object,
    );

    let ret = rrdb.pull_interval(report_id, time_start, time_end).unwrap();

    for (time, data) in ret.iter() {
        println!("data: {:x}, time {:x}", data, time);
    }
}


fn write_data(mut rrdb: RRDB, reportid_vec: Vec<ReportId>) {
    println!("Write mode start!");
    loop {
        for report_id in reportid_vec.iter() {
            let x1 = report_id.parameter as f64;
            let x2 = report_id.object as f64;
            let mem = mem_info().unwrap();
            let data: i64 = (100.0 * (x1.cos() + x2.sin())).floor() as i64 + mem.free as i64;
            rrdb.push_report(*report_id, data).unwrap();

            println!(
                "Data added - parameter: {:x},  object: {:x}, data: {:x}, time: {:x}",
                report_id.parameter,
                report_id.object,
                data,
                unix_time()
            );
        }

        let mem = mem_info().unwrap();
        println!(
            "Mem: total {} KB, free {} KB, avail {} KB, buffers {} KB, cached {} KB",
            mem.total, mem.free, mem.avail, mem.buffers, mem.cached
        );

        thread::sleep(Duration::from_secs(1));
    }
}


/// Making config elements and return reportid vec
fn make_config(rrdb: &mut RRDB, num_objects: usize, num_parameters: usize) -> Vec<ReportId> {
    let mut object_id = Vec::new();
    let mut reportid_vec = Vec::new();

    for num in 1..=num_objects {
        let object_name = format!("Chanel #{}", num);
        let mut object = Object::new(&object_name);
        let id = rrdb.push_config_element(&mut object);
        object_id.push(id);
    }

    for num in 1..=num_parameters {
        let parameter_name = format!("parameter #{}", num);
        let mut parameter = Parameter::new(&parameter_name, "test parameter");

        if num % 5 == 0 {
            parameter.aproxy_time = 5;
        }

        for id in object_id.iter_mut() {
            let reportid = ReportId {
                parameter: rrdb.push_config_element(&mut parameter),
                object: *id,
            };

            reportid_vec.push(reportid);
        }
    }

    reportid_vec
}


fn unix_time() -> u64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
}
