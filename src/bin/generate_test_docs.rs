use docx_rs::*;
use std::fs::File;

type DynError = Box<dyn std::error::Error>;

fn main() -> Result<(), DynError> {
    println!("Generating test documents...");

    std::fs::create_dir_all("tests/fixtures")?;

    generate_minimal_doc()?;
    generate_colors_doc()?;

    println!("All test documents generated successfully!");
    Ok(())
}

fn generate_minimal_doc() -> Result<(), DynError> {
    let doc = Docx::new()
        .add_paragraph(Paragraph::new().add_run(Run::new().add_text("Minimal Test").bold()))
        .add_paragraph(Paragraph::new().add_run(Run::new().add_text(
            "This is the smallest possible test document with just a title and one paragraph.",
        )))
        .add_paragraph(Paragraph::new().add_run(Run::new().add_text(
            "This single paragraph tests the most basic document parsing functionality.",
        )));

    let path = "tests/fixtures/minimal.docx";
    let file = File::create(path)?;
    doc.build().pack(file)?;
    println!("Generated: {path}");
    Ok(())
}

fn generate_colors_doc() -> Result<(), DynError> {
    let doc = Docx::new()
        .add_paragraph(
            Paragraph::new().add_run(Run::new().add_text("Color Formatting Test").bold().size(28)),
        )
        .add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text("Red text ").color("FF0000"))
                .add_run(Run::new().add_text("Green text ").color("008000"))
                .add_run(Run::new().add_text("Blue text ").color("0000FF"))
                .add_run(Run::new().add_text("Orange text ").color("FF8C00"))
                .add_run(Run::new().add_text("Purple text").color("800080")),
        )
        .add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text("Bold red").bold().color("FF0000"))
                .add_run(Run::new().add_text(" and "))
                .add_run(Run::new().add_text("italic blue").italic().color("0000FF")),
        )
        .add_paragraph(
            Paragraph::new().add_run(
                Run::new().add_text("This paragraph has no color formatting for contrast."),
            ),
        );

    let path = "tests/fixtures/colors.docx";
    let file = File::create(path)?;
    doc.build().pack(file)?;
    println!("Generated: {path}");
    Ok(())
}
