use terminal_size::{terminal_size, Height, Width};
use std::time::{Instant};

fn main() {
    let theta_spacing = 0.005;
    let phi_spacing = 0.005;

    let r1 = 0.5;
    let r2 = 1.0;
    let k2 = 5.0;

    let (screen_width, screen_height) =
        if let Some((Width(screen_width), Height(screen_height))) = terminal_size() {
            (screen_width / 2, screen_height)
        } else {
            (0, 0)
        };

    let k1: f64 = screen_width as f64 * k2 * 2.0 / (5.0 * (r1 + r2));

    println!(
        "Current settings: \n theta_spacing: {}, phi_spacing: {}, r1: {}, r2: {}, k2: {}, k1: {}",
        theta_spacing, phi_spacing, r1, r2, k2, k1
    );

    println!(
        "Current terminal size: width {}, height: {}",
        screen_width, screen_height
    );

    if screen_width.eq(&0u16) || screen_height.eq(&0u16) {
        return;
    }
    let mut a: f64 = 0.0;
    let mut b: f64 = 0.0;
    loop {
        if a.gt(&200.0) {
            break;
        }
        render_frame(
            a,
            b,
            screen_width,
            screen_height,
            theta_spacing,
            phi_spacing,
            r1,
            r2,
            k2,
            k1,
        );
        a += 0.1;
        b += 0.2;
    }
}

fn render_frame(
    a: f64,
    b: f64,
    screen_width: u16,
    screen_height: u16,
    theta_spacing: f64,
    phi_spacing: f64,
    r1: f64,
    r2: f64,
    k2: f64,
    k1: f64,
) {
    let start_time_point = Instant::now();

    let cos_a = a.cos();
    let sin_a = a.sin();
    let cos_b = b.cos();
    let sin_b = b.sin();

    let mut output = vec![vec![' '; screen_height as usize]; screen_width as usize];
    let mut z_buffer = vec![vec![0.0 as f64; screen_height as usize]; screen_width as usize];

    // theta goes around the cross-sectional circle of a torus
    let mut theta: f64 = 0.0;
    loop {
        if theta.gt(&(2.0 * std::f64::consts::PI)) {
            break;
        }
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();

        // phi goes around the center of revolution of a torus
        let mut phi: f64 = 0.0;
        loop {
            if phi.gt(&(2.0 * std::f64::consts::PI)) {
                break;
            }
            let cos_phi = phi.cos();
            let sin_phi = phi.sin();

            // the x,y coordinate of the circle, before revolving (factored
            // out of the above equations)
            let circle_x = r2 + r1 * cos_theta;
            let circle_y = r1 * sin_theta;

            let x =
                circle_x * (cos_b * cos_phi + sin_a * sin_b * sin_phi) - circle_y * cos_a * sin_b;
            let y =
                circle_x * (sin_b * cos_phi - sin_a * cos_b * sin_phi) + circle_y * cos_a * cos_b;
            let z = k2 + cos_a * circle_x * sin_phi + circle_y * sin_a;
            let ooz = 1.0 / z; // "one over z"

            // x and y projection.  note that y is negated here, because y
            // goes up in 3D space but down on 2D displays.
            let xp = (screen_width / 2) as f64 + k1 * ooz * x;
            let yp = (screen_height / 2) as f64 - k1 * ooz * y;

            // calculate luminance.  ugly, but correct.
            let l = cos_phi * cos_theta * sin_b - cos_a * cos_theta * sin_phi - sin_a * sin_theta
                + cos_b * (cos_a * sin_theta - cos_theta * sin_a * sin_phi);
            // l ranges from -sqrt(2) to +sqrt(2).  If it's < 0, the surface
            // is pointing away from us, so we won't bother trying to plot it.
            if l.gt(&0.0)
                // && xp.gt(&0.0)
                // && xp.lt(&(screen_width as f64))
                // && yp.gt(&0.0)
                // && yp.lt(&(screen_height as f64))
            {
                // test against the z-buffer.  larger 1/z means the pixel is
                // closer to the viewer than what's already plotted.
                let xp = xp as usize;
                let yp = yp as usize;
                if ooz.gt(&z_buffer[xp][yp]) {
                    z_buffer[xp][yp] = ooz;
                    let luminance_index = (l * 8.0) as usize;
                    // luminance_index is now in the range 0..11 (8*sqrt(2) = 11.3)
                    // now we lookup the character corresponding to the
                    // luminance and plot it in our output:
                    output[xp][yp] = char::from(".,-~:;=!*#$@".as_bytes()[luminance_index]);
                }
            }
            phi += phi_spacing;
        }
        theta += theta_spacing;
    }

    let fps = 1 as f64 / start_time_point.elapsed().as_secs_f64();
    for (pos, char) in fps.to_string().bytes().take(5).enumerate() {
        output[0][pos] = char::from(char);
    };

    // now, dump output[] to the screen.
    // bring cursor to "home" location, in just about any currently-used
    // terminal emulation mode
    print!("\x1b[H");
    for row in output.iter() {
        for item in row.iter() {
            print!("{}", item);
        }
        print!("\n");
    }
}
